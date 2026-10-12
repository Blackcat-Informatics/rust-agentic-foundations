# Completion audit: Stage 2 exit gate (fresh, independent Completion Adversary)

## Reviewed scope and limits

- Repo Blackcat-Informatics/rust-agentic-foundations. Issue 1, PR 11 (OPEN, MERGEABLE, head `df65ee4`).
- Branch `paudley/1-scaffold-workspace-project-docs-and`. Base `origin/main` @ `720262d`. Head `df65ee4`: 19 commits, all signed `G` and conventional, with no attribution trailers.
- The worktree was clean before the audit and is clean after it.
- Contract read in full:
  - `issue.md` (body and the 9 comments);
  - `issue-analysis.md`;
  - `plan.md`;
  - `remediation-plan.md`, with R1–R10 and its false-positive list;
  - `gap-analysis.md` (G1–G16);
  - `reviews/security-review.md`, `reviews/quality-review.md`, `reviews/completion-audit-stage2.md`;
  - `tasks/G-R1..R9-review.md`, read as needed;
  - `validation.md`;
  - `historical-fit.md`, through the gap-analysis adjudication.
- Tree read:
  - `Makefile`, both hooks, `ci.yaml` and `dependabot.yml`;
  - `tests/support/mod.rs` and `tests/repo_gates.rs`, in full;
  - `tests/docs_contract.rs`: inventory, checker driver, loader, fixtures module, and a body diff of every Stage 1 test;
  - AGENTS, CONTRIBUTING, SECURITY, CHANGELOG, README, LICENSING (Stage 2 diff), zh-Hans and the backtranslation record;
  - both architecture pages;
  - in full: all 9 census docs, source-index, okf-profile (capability), workplan-cli, workplan-reconcile, workplan-complete-gate, workplan-document-model and workplan-entity-history;
  - the Stage 2 diff of every other capability doc;
  - a section scan of all 5 considered docs.
- Authority:
  - Local build and test, and read-only commands, in the worktree.
  - Real signed commits only inside a scratch clone (`/tmp/raf-exit-audit.*`, removed afterwards). No hook was bypassed, and no `--no-verify` was used.
  - Read-only `gh` and `git ls-remote`.
  - No pushes, no GitHub mutations, no repo edits.
  - `CARGO_TARGET_DIR=/home/paudley/Active/.raf-stage2-target`.
  - One probe was blocked by a machine hook and not run: `make msrv` with a nightly `RUSTUP_TOOLCHAIN`. The msrv recipe is unchanged since the prior audit, which demonstrated that assertion.
- Raw evidence: `raw/exit-audit/`. It holds the probe logs, the 17 mutation scripts with their logs, and `probe.sh`/`mut.sh`.

## Binding user decisions

These are applied. The revised deliverable is distinguished from the original ask:

- **MSRV is 1.98.1** ("rust 1.98.1 is fine"). Issue text: 1.97.1. Tree: `Cargo.toml` `rust_version 1.98.1`; `MSRV_RUN ?= rust-validate 1.98.1`; CI `RUSTUP_TOOLCHAIN: 1.98.1` with a version assertion.
- **`.goals`, `.baseline` and `.stage/` are never committed.** This is in the issue body itself (issue.md:20). `git check-ignore -v` matches `.gitignore:6,7,5`. `git ls-files .goals .baseline .stage` is empty, and neither file exists on disk.
- **Census code may be ported without notice.** No crate code is in scope.
- **G3: "(1) but add an issue for (2)".** The Gate owns the census path. Issue 12 (created 2026-10-12T06:57Z, OPEN) holds the CLI mode as an enhancement.
  - This is not laundering. Option (1) is a complete, designed contract on its own, and the census-cli, rule-api, ledger, debt-ratchet and migration docs state it consistently.
  - Issue 12's scope adds a mode and removes nothing from issue 1.
- **"Workspace = build input (Recommended)"** and **"Fail-closed, honest (Recommended)"**. SECURITY.md and the specs state both.
- **"Port gates to Rust (Recommended)"**. Done: `tests/repo_gates.rs` and `tests/support/mod.rs`; the shell recipes are gone.

## Evidence reuse: full `make check`

`raw/make-check-df65ee4.log` qualifies as the full run for the current head, and I did not re-run it:

- Its header is `HEAD df65ee4`, and it ends `RC=0`.
- Every step is present and executed:
  - fmt;
  - clippy `-D warnings`;
  - test: docs_contract **112 passed**, repo_gates **40 passed**, lib 0, doctests 0;
  - doc `-Dwarnings`;
  - `cargo deny --offline`: advisories, bans, licenses and sources ok;
  - reuse 81/81;
  - `rust-validate 1.98.1 cargo check`;
  - licenses and issue-refs (repo_gates 40/40 each).
- The head is still `df65ee4`, and `git status` is clean.
- The nightly is still `rustc 1.100.0-nightly (4b6d04e70 2026-09-13)`.
- Cached artifacts went through cargo's fingerprint check. The tests in the log actually ran: they report non-zero counts.
- Independent spot re-runs on this tree:
  - `cargo test --locked --test docs_contract --test repo_gates` in the worktree: 112 and 40 passed.
  - In the scratch clone, every hook run re-executed fmt, reuse and both gate binaries on df65ee4-derived trees.

## Acceptance criteria

| # | Requirement | Demonstration | Observation | Status |
|---|---|---|---|---|
| AC1a | `make check` passes on the empty workspace | Reused `make-check-df65ee4.log` (above); `make -n check`; `cargo metadata --no-deps` | rc 0, all 9 steps. `make -n check` composes fmt, clippy, test, doc, deny, reuse, msrv, licenses, issue-refs. 1 package, `members = []`, 0 dependencies, empty stderr | **demonstrated** |
| AC1b | "…and so does CI" | `gh pr view 11 --json statusCheckRollup`; `gh run view 38225536340` (head df65ee4); check-run annotations | All 7 jobs (fmt, clippy, test, doc, deny, msrv, reuse) completed in 3 s with **0 steps**. Annotation: "The job was not started because recent account payments have failed or your spending limit needs to be increased." The 4 earlier Stage 2 heads show the same. Static checks only: `actionlint` rc 0, `shellcheck` rc 0, the `dtolnay` pin equals `refs/heads/master` `e2a55d2…` | **unverified: external blocker** (org billing). The local run is not a substitute |
| AC2 | `reuse lint` is clean | Reused log; `make reuse ROOT=<snapshot>` inside every scratch-clone hook run | Compliant 81/81 on df65ee4 and on every probe tree that did not plant an SPDX error | **demonstrated** |
| AC3 | Every listed capability exists with all 7 sections and resolved, current-tense text where the state allows | Independent python check of keys, state, id, fence-aware section order and change record for the 21 issue slugs; real-tree `docs_contract`; 17 mutation probes; full reading (see the Documents section); hedging-word scan of the designed docs | 21/21: exact 7 keys, `state` 9+7 `designed` and 5 `considered`, `id = cap:<stem>`, 7 sections in order, a change record each (0 bad). Every hedging-scan hit is conditional or normative. All 5 considered docs keep Contract, Usage, Examples, Constraints and Verification entirely inside PROPOSED blocks (0 plain lines). **Three designed-doc contradictions remain: F3, F4, F5** | **failing, narrowly** (resolved-text clause only: F3–F5); everything else demonstrated |
| AC4 | No issue numbers in repo docs; identities are slugs | `git grep -nIE '#[0-9]+'` over the whole tree; tracker-URL, `GH-`, `issue N` and `PR N` greps; process-wording grep; id = stem for every typed doc (docs_contract); hook probes | The only `#digits` hits are 6-digit hex colours (BRAND.md, the SVGs, a test fixture). No tracker forms. The process-wording hits are "remediation owner" in the `.deficiencies`/AGENTS ledger text and the SECURITY process. All ids are slugs. The tree meets AC4. **The gate that keeps it true is weaker than in Stage 1 (F1)** | **demonstrated** on the tree; enforcement regression F1 is open |

## Scope checklist

| Item | Demonstration | Observation | Status |
|---|---|---|---|
| Workspace manifest | `cargo metadata --no-deps`; tomllib | Edition 2024, resolver 3, `rust_version 1.98.1` (owner decision); licence `MIT OR Apache-2.0 OR MulanPSL-2.0`; authors `Blackcat Informatics Inc.`; repository URL; `publish=[]` | demonstrated |
| `[workspace.lints]`; reasons on allows; no rustflags | tomllib; read `Cargo.toml`; `git grep -i rustflags`; `ls .cargo` | Lints: deny `unreachable_pub`, `missing_docs`, `unsafe_code`; clippy all and pedantic warn; `allow_attributes_without_reason` deny. Each of the 2 allows has a reason comment above it. The rustflags grep is empty; there is no `.cargo/` | demonstrated |
| Toolchain floating nightly with clippy and rustfmt; no `#![feature]` | tomllib; `git grep '#!\[feature' -- '*.rs'` | `channel=nightly`, both components; the grep is empty | demonstrated |
| rustfmt, clippy and deny configs | Unchanged since d6b6572; `cargo deny --offline` (log) | Present; deny ok | demonstrated (reuse plus log) |
| Makefile `check` | `make -n check`; `make reuse REUSE=/nonexistent/reuse`; `make msrv MSRV_RUN=/nonexistent/rv` | All issue steps plus msrv, licenses and issue-refs. Each missing tool: rc 2, with the tool named | demonstrated |
| `.githooks/pre-commit` checks the staged index; `make hooks` | Scratch clone: `make hooks` changes `core.hooksPath` from unset to `.githooks`; signed commit probes (below) | The hook judges the index: a token staged with a clean working tree is refused. It runs fmt, reuse and **both Rust gate binaries** (`Running tests/docs_contract.rs`, `Running tests/repo_gates.rs` in the logs). The positive control commit `05fd1cb G` was accepted | demonstrated (not DARK) |
| `pre-merge-commit` (G16-F5) | Scratch clone merges (below) | A non-conflict merge carrying a token was refused. A non-conflict merge of two individually valid branches that breaks a cross-file anchor was refused. A clean merge produced a 2-parent signed commit | demonstrated |
| `ci.yaml` (fmt, clippy, test, doc, deny, msrv) | `actionlint`; `yq '.jobs\|keys'` | Jobs fmt, clippy, test, doc, deny, msrv, reuse. The repo gates run inside `test`. Pins: checkout v7.0.0, setup-uv v10.1.0 with uv 0.13.0 and `enable-cache: false`, `cargo-deny@0.20.2`, dtolnay = master head. Concurrency group set | file demonstrated; **execution unverified** (AC1b) |
| AGENTS, CLAUDE, `.deficiencies`; `.goals`/`.baseline`/`.stage` ignored | Read the files; `git check-ignore`; `git ls-files` | All present and tracked; ignore rules match; nothing internal is tracked | demonstrated |
| origin URL | `git remote get-url origin` | `ssh://git@ssh.github.com:443/Blackcat-Informatics/rust-agentic-foundations.git` | demonstrated |
| LICENSING.md | Read the Stage 2 diff | Code contributions under the three licences; documentation contributions under CC-BY-4.0 (G14e) | demonstrated |
| Licence texts, Mulan byte pin | `sha256sum`; hook probe with an altered LICENSE-MULAN | Both Mulan files hash to `eb7a1d71…`, CC-BY to `d557539d…`. An altered LICENSE-MULAN was refused by name (`licenses_mulan_texts_match_the_pinned_digest`) | demonstrated |
| zh-Hans plus backtranslation | Char-level difflib against the sed-renamed purrdf text; `sha256sum`/`wc -c` against the record; hook probe with an altered zh file | One insertion (`®`) and 5 deletions only. No new Chinese, and the sentences stay grammatical. `第三方` ×2; `来源记录` absent. Record: 1,712 bytes, SHA `e536de48…` = the file. An altered zh file was refused (`licenses_zh_hans_matches_the_recorded_fingerprint`) | demonstrated |
| CONTRIBUTING | Read in full | All required sections. Advisory V1: "bound tests pass" omits examples | demonstrated |
| CODE_OF_CONDUCT | Unchanged since d6b6572; prior audit diff | Name-only adaptation | demonstrated (reuse) |
| SECURITY | Read; hashed the PGP block in this repo and in purrdf; `gpg --show-keys` with a throwaway keyring | Block `6bb700ee…` = purrdf. Fingerprint line `52E5 4B13 … 8F9A 0090` = gpg output. UID note present. Trust model and the hardened contracts are stated; the attack surface covers all five named items | demonstrated |
| Issue and PR templates | Unchanged except the PR template's SPDX split (R5); SPDX-shape scan | Valid | demonstrated |
| CITATION, cliff, CHANGELOG | `uvx cffconvert --validate`; `git cliff --unreleased` (worktree); scratch-clone dedup scenario | CFF valid (1.2.0). git cliff rc 0. Dedup: the new config prints `**BREAKING** **census:** Add the JSON report` once; the d6b6572 config prints it twice | demonstrated |
| SPDX everywhere; `REUSE.toml`; reuse lint | Shape scan over all tracked `*.md`; reuse | Every Markdown file is CC-BY-4.0 with the 2-line HTML header or YAML comments; reuse 81/81 | demonstrated |
| BRAND.md | Unchanged since 49eb42b; prior audit read | Name, ids, positioning, tagline, description | demonstrated (reuse) |
| cat-head-core verbatim; accents only in `service-toolbox` | Python extraction of both SVGs; XML ancestor walk | The group text is byte-equal to purrdf's. No accent use outside `service-toolbox` | demonstrated |
| Logo, social SVG and PNG (rsvg 1280×640) | `rsvg-convert -w 1280 -h 640` then `cmp`; IHDR; `xmllint` | Byte-identical, 1280×640, XML ok | demonstrated |
| README | Read; links checked by docs_contract (including `<img src>`, probe m13) | Crates marked designed, not yet published; "Planned usage" snippets; `raf-census = "0.1"` | demonstrated |
| 9 census caps (`designed`) | AC3 check | 9/9 | demonstrated (content: F3, F4) |
| 7 workplan caps (`designed`) | AC3 check | 7/7 | demonstrated (content: F5) |
| 5 M2 caps (`considered`) with change records | AC3 check; plain-line scan | 5/5. Candidates sit only inside PROPOSED blocks | demonstrated |
| Architecture pages | Read both | normal-form is unchanged; okf-profile carries the section table, the state table and the containment rule | demonstrated (content: F2) |
| Essay moved and file removed | `test ! -e INITIAL_PLANNING_PLAN.md`; `git grep INITIAL_PLANNING`; G1 greps | Absent and unreferenced. Section table, GROUNDING URL ×2, and "accurate capability explanation" in Scenario B are all present | demonstrated |

## Stage 2 gaps and findings: fixed or evidenced false positive

| Gap or finding | Disposition checked in the tree | Status |
|---|---|---|
| G1 (essay losses) | okf-profile.md:89-97 section table; GROUNDING links in document-model:214 and two-way-coverage; complete-gate:206-207 | FIXED |
| G2 / AC1b (CI) | Billing block persists on df65ee4 | external blocker, unverified |
| G3 (CLI vs Gate) | census-cli:49-61 and 113-119; rule-api:60-70 and 108-124; ledger:80-90; debt-ratchet:52-58; migration:55-68 | FIXED per decision (1); issue 12 holds (2) |
| G4 (examples at completion) | reconcile:48 `example-failing`; complete-gate:35-41; "criterion" removed | FIXED |
| G5 (depends edges) | All listed edges present; `raf_source::Tree` gone from the okf-profile capability; the `depends` meaning is stated (arch:64-65); the graph is acyclic (docs_contract) | FIXED |
| G6 (affects ⊇ PROPOSED sections) | 5 records list `#constraints`; rule enforced (probe m16) | FIXED |
| G7 (shell gates) | Rust gates; one pattern site (`grep -c 'this\[ \]issue'` gives 1, in `tests/support/mod.rs`); the hook and the Makefile run them | FIXED; but see F1 |
| G8 (bindings by state) | arch:30-36 qualified; rule enforced (probe m12) | FIXED |
| G9 (hex and entity false positives) | Fixtures pass for `fill="#000"`, `&#38;`, `#1f2`, `#123456` | FIXED, **over-broad (F1)** |
| G10 (cliff dedup) | Scratch scenario, before and after | FIXED |
| G11 / S-L1 (tool pins) | `cargo-deny@0.20.2`, uv 0.13.0, `enable-cache: false` | FIXED |
| G12 (delivered minimum) | Identical wording at arch:111, document-model:104 and complete-gate:59; constraint reworded | FIXED |
| G13a–g (census nits) | a `--frozen`; b `--ledger` default excluded; c the Reads column (9 shipping, 5 all); d unreferenced pattern fails; e "every key except `[job.ext]`"; f empty positive set refused; g bump path stated | FIXED; g leaves contradiction F4 |
| G14a–e (licensing) | See the zh row above | FIXED |
| G15a–d (project docs) | `"0.1"`; CHANGELOG bullets; PR-template SPDX shape; AGENTS §6 | FIXED |
| G16 F4, F5, F7, F9 | Concurrency; pre-merge-commit (demonstrated); citation agreement (hook refused a CITATION 0.2.0 bump); dependabot | FIXED |
| S-M1 (containment) | source-index:32-47, 135-145, 216-231; reference-scan:67-81; arch:157-163 | FIXED in the specs; arch text stale (F2) |
| S-M2 (trust and invocations) | SECURITY trust model; exact argv in reference-scan, workplan-cli, complete-gate, reconcile and entity-history; `--frozen` everywhere; `--no-deps` for root discovery | FIXED |
| S-L2 (dtolnay pin) | `ls-remote` master = `e2a55d2…` = the pin | FIXED |
| S-L3 (PGP) | Fingerprint and UID note; renewal before 2026-12-24 is an owner action | FIXED (owner action remains) |
| S-L4 (parser bounds) | okf-profile capability:38-47; ledger:32-34; source-index:49-73 | FIXED |
| S-L5 (ROOT quoting) | Makefile:23 and 43 quote `'$(ROOT)'` | FIXED |
| Q-H1 (inventory) | `INVENTORY`; probes m01 (unlinked delete), m02 (delete plus link scrub), m03 (consistent rename) and m17 (state) all fail | FIXED |
| Q-M1 to Q-M5, Q-L2, Q-L3, Q-L6, Q-L7 | Probes m04–m16 each fail with the named rule; the hook runs docs_contract (probe "broken capability") | FIXED |
| Q-L1, Q-L4, Q-L5, Q-L8; historical F6, F8, D3; A5; F7 second half | Evidenced false positives or declines, accepted as the remediation plan records them | not gaps |
| Prior audit advisories A1–A4 | CHANGELOG, version, zh third-party sentence, memory-bound sentence | FIXED |
| Prior audit advisories A6 and S-L3 renewal | Social preview upload, repository description, key renewal | owner actions |

## Failure-shape review

- **DARK: none.** `repo_gates` and `docs_contract` are reached by `make test`, `make check`, the pre-commit and pre-merge-commit hooks (demonstrated), and the CI test job (configured). The real-tree tests use the same `check`/`check_inventory` and `banned_in_line` that the fixtures use.
- **PRODUCER-WITHOUT-CONSUMER: none.** The backtranslation fingerprint is consumed by `licenses_zh_hans_…`. `INVENTORY` is consumed by the real-tree test. `cliff.toml` is consumed by `git cliff`. The PNG's consumer is the manual Settings upload, which is an owner action.
- **TEST-ONLY: none.** Every claim above was exercised through `make`, the hooks under real signed commits and merges, `cargo test` on the real or mutated tree, `git cliff` or `rsvg-convert`.
- **WEAKENED TESTS: none in docs_contract, one gate regression (F1).**
  - All 46 Stage 1 docs_contract test names survive. 44 bodies are byte-identical.
  - The 2 changed bodies are strengthened:
    - `repository_documents_satisfy_the_profile` adds the inventory and an unconditional Capability/Change guard;
    - `accepts_moved_outcome` now has to supply the target record that the new `moved` rule requires.
  - The fixture-helper changes (`capability()` puts the body inside Purpose; delivered fixtures get bindings) keep the old assertions intact.
  - No `#[allow]`, no `#[ignore]`.
  - The ported issue-reference matcher is narrower than both Stage 1 implementations: F1.
- **SILENT NARROWING: F1** (the gate scope). There is none in the capability set or the scope items.
- **LAUNDERED-TO-FOLLOWUP: none.** Issue 12 follows the user's decision and covers an added mode.
- **DUPLICATE: none.** The shell gates and the Stage 1 Rust duplicate are removed, and the pattern set has one definition site.
- **`.deficiencies`** (worktree root): the notice and marker, with nothing below the marker. The main checkout root has no `.deficiencies` file.

## Documents: hedging, contradiction, PROPOSED discipline

- **No hedging or unresolved placeholders in the designed docs.**
  - The Gate-owns contract is stated the same way in all five census docs.
  - The security contracts state their limits honestly, per the "Fail-closed, honest" decision: for example source-index:64-72 and SECURITY "Bounds".
  - Considered docs: Purpose and open questions are plain text, and every other non-Explanation line is inside a PROPOSED block.
- **Three contradictions in designed docs** (F3–F5), plus one stale normative claim on an architecture page (F2). Details below.

## Open findings

1. **AC1b: hosted CI has never executed (external blocker, UNVERIFIED).**
   - Evidence: run 38225536340 on df65ee4; all 7 jobs show 0 steps and the billing annotation.
   - Required: the owner clears billing, the workflow is re-run on the final head, every job reaches a real conclusion, and the result is recorded in `validation.md`. Merge needs green CI or an explicit owner waiver.

2. **F1 (MEDIUM): the ported issue-reference gate no longer catches `owner/repo#N` or `word#N`. This is a WEAKENED gate.**
   - Where: `tests/support/mod.rs:176-192` (`hash_number`). Line 179 spares any `#` preceded by a word character.
   - Claims it falsifies:
     - `CONTRIBUTING.md:73-74`: "fails on issue or pull request numbers … in any tracked text file";
     - `CHANGELOG.md:24-26`: "scans every tracked file for issue and pull request references";
     - `AGENTS.md:193-198`.
   - Demonstration: in the scratch clone, `docs(census): audit probe cross-repository reference` (`394abb4`, signed `G`) was **accepted by the pre-commit hook** (112/112 and 40/40 passed). It adds `Blackcat-Informatics/rust-agentic-foundations#12 and in rust-lang/cargo#1234` to `docs/capabilities/census-cli.md`.
   - On the same staged line, the Stage 1 pattern `git grep --cached -nIE '#[0-9]{1,5}\b'` hits. The Stage 1 Rust `hash_number` (d6b6572 `tests/docs_contract.rs:378-391`) has no prefix exemption either.
   - Consequence:
     - GitHub's canonical cross-reference form, which GitHub renders as a link to issue 12 of this repository, can enter any tracked file through the hook, `make check` and CI.
     - The AC4 / `.baseline` rule is weaker than in Stage 1.
     - The tree is clean today, and no tracked file contains `\w#\d`, so the exemption serves no current file.
   - Required fix:
     - Drop the word-character exemption.
     - If a numbered-heading anchor such as `AGENTS.md#3-gates` must stay legal, spare `#N` only inside a Markdown link target, or only when the digits are followed by `-`.
     - Add negative fixtures `org/repo#12`, `repo#12` and `Fixes#12`, plus a passing anchor fixture if the anchor form is kept.
     - Keep the fixtures that already pass.

3. **F2 (LOW): the normative profile describes a checker mechanism that no longer exists.**
   - Where: `docs/architecture/okf-profile.md:161-163`: "looked up in the set of repository file paths that the walk collected, and the walk skips symlinks and non-regular files, so a symlink cannot lead a link outside the root".
   - R3 wrote this text. R6 then replaced the walker with `git ls-files` (`tracked_paths`), and the text was never updated, although G-R3 advisory A-7 asked for alignment.
   - Demonstration: in the scratch clone, I staged a tracked symlink `docs/outside-link.svg -> /etc/hostname` and a README link to it.
     - `cargo test --test docs_contract`: **112 passed**. The link is accepted, because the symlink is in the path set.
     - Only `repo_gates` refuses it ("symlinks are not allowed in the tree").
   - Consequence: the stated property holds only at suite level, and the page misdescribes the enforcement it calls normative.
   - Required fix: reword the sentence to "looked up in the set of tracked paths (the git index); a tracked symlink or non-regular file is refused by the repository gates". Alternatively, make `check_links` refuse a target that is a tracked symlink, with a negative fixture.

4. **F3 (LOW, AC3): `sole-name` contradicts the home-package rule.**
   - Where: `docs/capabilities/census-detectors.md:40`. `sole-name` reports "a function **inside the home crate** …".
   - It conflicts with `:58-59`, "A detector never reports a hit inside the home package of the job that forbids it", and with Verification `:137-138`.
   - Read together, `sole-name` can never fire. This predates Stage 2, and G13c edited the same paragraph.
   - Required fix: except `sole-name` from the sentence. For example: "Apart from `sole-name`, which exists to report inside the home crate, a detector never reports …". Add or adjust the matching Verification bullet.

5. **F4 (LOW, AC3): the bump instruction relies on a migration the migrate contract refuses.**
   - Where: `docs/capabilities/census-migration.md:70-72` (added in Stage 2 by G13g) and `census-debt-ratchet.md:75-77`. Both say to run `cargo census migrate` on the ledger at a `normal_form` bump.
   - The designed `migrate` contract does not do this:
     - it reads "a legacy baseline and a legacy ledger" (`:27-29`);
     - "The output sets `normal_form = 1`" (`:36`);
     - it "knows the legacy ledger layouts of the earlier census copies and rejects any other layout" (`:124-125`).
   - A unified ledger at an earlier RAF `normal_form` is not among those layouts, and the doc never says where the "old baseline" comes from at a bump.
   - G-R2 noted the gap as an advisory, and the fix named a command without reconciling the contract.
   - Required fix, one of:
     - (a) State in Contract and Constraints that `migrate` also reads a unified ledger at the previous `normal_form`, with the baseline report that the previous version wrote (`cargo census baseline --json` before upgrading), and writes the implemented version.
     - (b) Reword Usage and the debt-ratchet bump path to say that a `normal_form` bump extends `migrate` with the mapping from the previous version, as part of the bump change.

6. **F5 (LOW, AC3): a constraint claims "exactly one test ran" for every binding.**
   - Where: `docs/capabilities/workplan-complete-gate.md:161-163` (added in Stage 2 by R3): `complete` "counts a binding as passed only when it proved that exactly one test ran".
   - The same doc defines other rules:
     - doc-test bindings, where every entry the list pass selects must pass, and that can be more than one (`:99-115`);
     - example targets, judged by their own exit status (`:117-119`).
   - Required fix: scope the bullet to test bindings, or restate it as "… only when it proved that the bound test, doc tests or example actually ran, by the rules above".

## Advisories (non-blocking)

- **V1.** `CONTRIBUTING.md:42` says completion needs "the bound tests pass". AGENTS.md:185 and complete-gate condition 5 say tests **and examples**.
- **V2.** Stale comments in `tests/repo_gates.rs`:
  - `:17-19` says test-name prefixes let `make licenses`/`make issue-refs` "select a gate", but the Makefile deliberately runs the whole binary.
  - `:226-228` calls invalid UTF-8 binary, but `decode_text` scans it lossily.
- **V3.** `census-debt-ratchet.md:69-71` calls file creation "the one case" of recording debt with no earlier record. The bump replacement at `:75-78` is a second case; the Explanation at `:130-133` already names both.
- **V4.** `workplan-document-model.md:46-48 and 138-148` show a delivered capability under revision with PROPOSED blocks but never name its state. Under the profile (designed has no PROPOSED blocks; delivered has no open record), only `implementing` fits. Say so.
- **V5.** A `docs_contract` frontmatter failure is reported twice: once by `check` and once by `check_inventory`. This is cosmetic (probe m06).
- **Owner actions.** Clear CI billing. Upload `docs/social-preview.png` and set the repository description. Renew the PGP key before 2026-12-24.

## Verdict

**FINDINGS-OPEN.**

The CI half of AC1 is unverified because of the external billing block. Beyond it, **not every other criterion is fully demonstrated**:

- AC1a, AC2 and AC4 (on the tree) are demonstrated.
- Every Scope checklist item is demonstrated or, for the CI file, statically verified.
- Every Stage 2 gap is FIXED in the tree or is an evidenced false positive.
- Remaining work:
  - **F1** (MEDIUM): restore the issue-reference gate's coverage of `owner/repo#N`, with fixtures.
  - **F2–F5** (LOW): four one-to-two-sentence document corrections. F3–F5 hold AC3's "resolved text" clause.

---

## Recheck (pass 2)

### Scope and limits

- Head `edf93d5`. It adds R11 on top of the pass-1 head `df65ee4`:
  - `45bc98e` test: catch repository-qualified issue references again;
  - `edf93d5` docs: resolve contradictions in the census and workplan contracts.
- Both commits are signed `G`. The worktree is clean.
- Reviewed:
  - the whole R11 diff (`git diff df65ee4..edf93d5`, 13 files, +202/−76);
  - `tasks/G-R11-review.md`;
  - `raw/make-check-edf93d5.log`;
  - `validation.md` lines 87-91.
- Not re-audited: criteria that R11 did not touch and that pass 1 already demonstrated.
- Qualification: `raw/make-check-edf93d5.log` is reused and qualifies:
  - the header is `HEAD edf93d5`, and it ends `RC=0`;
  - every step ran: docs_contract **113 passed**, repo_gates **42 passed**, deny ok, reuse 81/81, msrv, licenses, issue-refs;
  - the head is still `edf93d5`, and the tree is clean.
- Probes ran in a scratch clone of `edf93d5`, after `make hooks`, with real signed commits and no bypass. The positive control `c8975ee` was accepted (113 and 42 passed). The clone has been removed, and the logs are in `raw/exit-audit/recheck-pass2/`.

### Per-finding disposition

| Finding | Recheck | Status |
|---|---|---|
| AC1b, hosted CI | Still refused by the billing block | **STILL-OPEN, external blocker, unverified** |
| F1, `owner/repo#N` gate gap | I re-ran the pass-1 probe unchanged: `Blackcat-Informatics/rust-agentic-foundations#12` and `rust-lang/cargo#1234` appended to `census-cli.md` as a signed commit. **The hook refused it.** Both binaries reported it: `repo_gates` with `census-cli.md:124: issue or pull request number` and `docs_contract` with `line 124: issue or pull request number is banned`.<br>The variants `Fixes#12`, `repo#12`, `Fixes:#123` and `see:#1234`, committed in `src/lib.rs`, were each refused.<br>In `tests/support/mod.rs`, the word-character and `:` exemptions are gone, and the doc comment matches the code.<br>New fixtures cover `org/repo#12`, `repo#12`, `Fixes#12`, `x#1`, the full cross-repository form, `Fixes:#123`, `id:#1234` and `color: #123`. The one renamed fixture keeps both old assertions and adds two more, so nothing was weakened. The real tree passes | **DISCHARGED** |
| F2, stale symlink and link text | I re-ran the pass-1 probe unchanged: a tracked `docs/outside-link.svg -> /etc/hostname` and a README link to it. **`docs_contract` alone now fails** with ``README.md: line 75: link `docs/outside-link.svg` targets a tracked symlink``.<br>The signed commit was refused, by both binaries.<br>An `<img src>` pointing at a tracked symlink is refused as well (`README.md: line 5`).<br>`okf-profile.md:160-163` now describes the lookup in the tracked-path list and the symlink violation accurately. Fixture: `rejects_link_to_a_tracked_symlink` | **DISCHARGED** |
| F3, `sole-name` | `census-detectors.md:58-60` states `sole-name` as the exception. The Verification bullets at `:138-141` are split to match | **DISCHARGED** |
| F4, bump vs migrate contract | The contract now accepts "a ledger in the current format whose `normal_form` is older than the implemented version" and writes the implemented `normal_form`. This is consistent across the description, Purpose, Contract, Usage, Constraints and Verification, `census-cli.md:42,47,143`, `census-debt-ratchet.md:68-80`, `normal-form.md:137-143` and the change record. The contradiction pass 1 named is gone. **The fix introduces N1** (below) | **DISCHARGED** (as stated) |
| F5, proof of a run | `workplan-complete-gate.md:161-166` states the proof per binding kind: one test ran; every selected doc test reported `ok`; the example exited 0 within the timeout. The "Proof" paragraph is scoped to test bindings. This agrees with `:92-119` | **DISCHARGED** |
| V1–V4 | CONTRIBUTING says "tests and examples". The repo_gates comments match the Makefile and `decode_text`. The debt-ratchet "Two cases" text agrees with its Explanation. A revised delivered capability "moves it to `implementing`", which fits the profile table and `docs_contract` | done |
| V5 | Declined, with the reason at `validation.md:89`: cosmetic, and the gate still fails closed | accepted |

### New finding from the R11 text

**N1 (LOW, AC3 "resolved text"): the extended migrate path uses a report that drops the fingerprints it must map, and its carry-over rule omits the current-format tables.**

**(a) The input report drops single-member groups.**
- For a current-format ledger, the text makes the older-version input "the report that `cargo census baseline --json` wrote under the older `normal_form`". Places:
  - `census-migration.md:33-35`;
  - Usage `:61-66` and `:83-85`;
  - Constraints `:128-130` and `:143-144`;
  - Verification `:158-160`;
  - `census-debt-ratchet.md:76-78`.
- The census specs define that report as groups only:
  - "A group has two or more members" (`census-normal-form.md:52`);
  - "The baseline report lists every group" (`:56`);
  - "groups of one are dropped" (`:175`).
- `census-ledger.md:103` requires each forbidden `body`/`shim` fingerprint to match a unit. In a clean workspace that unit is the home alone, which is a group of one and so not in the baseline report.
- At a bump, every such row hits "the baseline has no group for the fingerprint" (`census-migration.md:46-48`). It is listed as unmapped, the exit code is 1, and the fingerprint survives only as a comment.
- The bump path therefore cannot rebuild a clean ledger, which is the state the ratchet drives toward. That contradicts the Purpose ("without re-deriving its ledger by hand").
- The mapping the path needs is in the index report, "every unit with its fingerprints" (`census-normal-form.md:56-57`, `cargo census index`).

**(b) The carry-over rule omits the current-format tables.**
- `census-migration.md:38-43` maps "each old fingerprint named in the old ledger" through the baseline, and keeps only "`min_tokens`, `roots` and the job and variant rows".
- For a current-format ledger it does not say that these pass through unchanged:
  - the name-kind ids, `table:<name>` and `rule:<id>`, which "carry names, not hashes" (`census-normal-form.md:139`);
  - `[[pattern]]`, `[[distinct]]`, `reference` and `max_file_bytes`.
- Read literally, a `rule:` id has no baseline group and is unmapped.
- A `[[pattern]]` that is not kept leaves a job's `forbidden.patterns` naming an unknown pattern, which fails the ledger (`census-patterns.md:135`).

**Required fix.**
- For a current-format ledger, name the index report written before the upgrade (for example `cargo census index > index-before-upgrade.json`) as the input. Alternatively, define a baseline output that keeps single-member groups.
- State that `table:`/`rule:` ids and the fingerprint-free tables and keys are carried over unchanged.
- Add Verification fixtures, each migrating with no unmapped row:
  - a clean enforced job whose fingerprint only the home carries;
  - a job forbidding a `rule:` id;
  - a ledger with a `[[pattern]]` and a `[[distinct]]` row.

### Regression review of the R11 diff (advisory, non-blocking)

- **R-a.** A tracked top-level `.md` symlink, with nothing linking to it, now passes `docs_contract` alone (113 passed). The loader substitutes `SYMLINK_TEXT` where `df65ee4` panicked in `read_tracked`. The suite still refuses it: the hook refused `NOTES.md -> /etc/hostname` through `repo_gates` ("symlinks are not allowed in the tree"), and `make test` and the CI test job run both binaries. No claim is falsified.
- **R-b.** The new Usage example at `census-migration.md:64` uses `--ledger census.toml --out census-new.toml`. The ledger is the file `helpers-ledger.toml` (`census-ledger.md:28`). Use that name, and say that the output replaces it.
- I found no other regression:
  - the issue-reference change makes the gate stricter and fails closed (`color: #123`);
  - the tree passes;
  - no Stage 1 or Stage 2 test lost an assertion.

### Verdict (pass 2)

FINDINGS-OPEN. N1 (LOW) remains. The F4 fix introduced it, and it holds the "resolved text" clause of AC3. Hosted CI, the second half of AC1, is unverified because of the external billing block. F1, F2, F3, F4 (as stated) and F5 are DISCHARGED. Every other in-scope criterion is demonstrated.

---

## Recheck (pass 3)

### Scope and limits

- Head `80c14d7` (signed `G`; clean tree): `docs(census): map fingerprints through the index report at a normal-form bump`.
- Reviewed:
  - the whole delta `git diff edf93d5..80c14d7`: 5 files, +52/−19. They are `census-migration.md`, `census-debt-ratchet.md`, `census-cli.md`, `work/changes/census-migration.md` and `tests/docs_contract.rs`;
  - `tasks/G-R11-review.md` "Recheck 2";
  - `raw/make-check-80c14d7.log`.
- Not re-audited: criteria outside that delta that earlier passes demonstrated. The delta does not touch `tests/support/mod.rs`, `repo_gates`, the hooks, the Makefile or CI. I still re-ran the F1 probe as a regression check.
- Qualification: `raw/make-check-80c14d7.log` is reused and qualifies:
  - the header is `HEAD 80c14d7`, all 9 step headers are present, and it ends `RC=0`;
  - docs_contract **114**, repo_gates **42**;
  - deny ok, reuse 81/81.
- Probes ran in a scratch clone of `80c14d7` after `make hooks`, with real signed commits and no bypass. The clone has been removed, and the logs are in `raw/exit-audit/recheck-pass3/`.
  - Positive control `fba136b G`: accepted (114 and 42 passed).
  - The real tree tracks 0 symlinks.
- Hosted CI on `80c14d7` (run 38229506850): all 7 jobs show 0 steps, with the billing annotation.

### Disposition

| Item | Recheck | Status |
|---|---|---|
| N1(a): the bump input report | `census-migration.md:33-38`: for a current-format ledger, `--old-baseline` is the **index report** (`cargo census index`) that the older version wrote. The baseline report is excluded, and the text explains why (it drops one-member groups, which is how a clean job's fingerprint is carried). `:42-43`: "the members of a fingerprint are the units that list it". This matches `census-normal-form.md:56-57` and the index sample at `:116-125`, which lists every unit with `body` and `shim`. A clean job's home-only fingerprint is therefore found and remapped. Usage `:73-77` and `:98`, Constraints `:143-145` and `:158-159`, Verification `:173-180`, `census-debt-ratchet.md:77`, the `census-cli.md:42` row and the change record all name the index report. `git grep 'baseline report' -- docs work` shows no remaining bump-path use; the hits are the `baseline` subcommand, the report definition and the exclusion sentence | **DISCHARGED** |
| N1(b): pass-through | `census-migration.md:50-53`: for a current-format ledger only `body:` and `shim:` are remapped. `table:`/`rule:` ids, `[[pattern]]`, `[[distinct]]`, `reference` and `max_file_bytes` carry over unchanged. This agrees with `census-normal-form.md:139` (name kinds carry names) and with the census-ledger tables. A Verification bullet covers the clean job, the `rule:` id, and the `[[pattern]]` and `[[distinct]]` rows with no unmapped row | **DISCHARGED** |
| Note R-a: an unlinked tracked symlink | `tests/docs_contract.rs` `check()` now reports every `SYMLINK_TEXT` entry and excludes it from the link, scan and marker passes. Probe: tracked `NOTES.md -> /etc/hostname`. **`docs_contract` alone fails** (`NOTES.md: a tracked symlink is not allowed`, 113/114), and the signed commit is refused by both binaries. Fixture: `rejects_a_tracked_symlink_that_nothing_links_to` | **DISCHARGED** |
| Note R-b: Usage ledger name | `census-migration.md:73-77` uses `--ledger helpers-ledger.toml --out helpers-ledger.new.toml` and says to move the output over the ledger. The legacy example gains a matching sentence at `:70-71` | **DISCHARGED** |
| Regression: F2 | Tracked `docs/outside-link.svg` symlink plus a README link to it. `docs_contract` reports both `README.md: line 75: link … targets a tracked symlink` and `docs/outside-link.svg: a tracked symlink is not allowed`. Link resolution still sees symlink entries | no regression |
| Regression: F1 | `Blackcat-Informatics/rust-agentic-foundations#12` in `census-cli.md`, as a signed commit, is refused by `docs_contract` and `repo_gates` | no regression |
| Regression: legacy migrate path | The legacy Contract bullets, Usage and unmapped behaviour are unchanged in substance. The pass-through rule is scoped to "a current-format ledger" | no regression |
| Regression: tests | No assertion was removed or weakened. One fixture was added. The 0-symlink real tree passes, so there are no false positives | no regression |

### Advisories (non-blocking)

- `census-migration.md:75-76` puts `cargo census index > index-before-upgrade.json` straight before the `migrate` line. The text above it says the report is "written before the upgrade". A short note that the first command runs on the earlier RAF version would prevent a misreading. A newer version cannot silently produce the wrong file, because a ledger at an older `normal_form` fails hard (`census-normal-form.md:58-60`).
- The pass-through reason "holds names and not hashes" is loose for a `tokens` `[[pattern]]`: its token sequence is compared under the normal form's normalisation. Carrying it over unchanged is still correct, and the pattern staleness rule (`census-patterns.md:46-47`) reports one that stops matching.

### Final verdict

Every prior finding is DISCHARGED: F1–F5, N1, and the notes R-a and R-b. No regression was found. Every in-scope criterion is demonstrated with execution evidence on `80c14d7`:

- AC1a: `make check` rc 0 on `80c14d7`;
- AC2: reuse 81/81;
- AC3: all 21 capabilities with resolved designed text, and no open contradiction;
- AC4: the tree is clean, and the gate is restored;
- every Scope checklist item and every Stage 2 gap.

The one exception is the hosted-CI half of AC1, which is **unverified because of an external blocker**. Every CI run on this branch, including run 38229506850 on `80c14d7`, was refused by GitHub billing before any step ran. It needs an owner billing action, then a green run on the final head, or an explicit owner waiver for merge. A local run is not a substitute for it.

**VERDICT: PASS. Every in-scope criterion is demonstrated except the hosted-CI half of AC1, which is unverified because of the external billing block.**
