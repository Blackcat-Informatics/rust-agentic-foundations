# Issue #1: Scaffold workspace, project docs and branding, plus spec-first capability docs

State: OPEN   Repo: Blackcat-Informatics/rust-agentic-foundations   Forge: github
Labels: documentation, repo, dogfood, spec

## Body

## Context
This issue lays the foundation for `rust-agentic-foundations` (RAF), a set of publishable crates that Blackcat projects use as dev-dependencies. It also starts the dogfooding: the design of both tools is written **first**, as capability documents in the workplan format that `raf-workplan` will later enforce.

## Scope
### Workspace
- [ ] `Cargo.toml` workspace: edition 2024, resolver 3, `rust-version = "1.97.1"`, shared `[workspace.package]` (authors `Blackcat Informatics Inc.`, license `MIT OR Apache-2.0 OR MulanPSL-2.0`, repository URL)
- [ ] `[workspace.lints]` (logit-loom set: `missing_docs`, `unreachable_pub`, `unsafe_code` = deny; clippy `all` + `pedantic` warn; every allow carries a reason). No rustflags.
- [ ] `rust-toolchain.toml`: floating nightly with clippy and rustfmt; no `#![feature]` anywhere
- [ ] `rustfmt.toml`, `clippy.toml`, `deny.toml`
- [ ] `Makefile` with a `check` target (fmt, clippy `-D warnings`, test, doc, deny, `reuse lint`)
- [ ] `.githooks/pre-commit` (checks the staged index) and `make hooks`
- [ ] `.github/workflows/ci.yaml` (fmt, clippy, test, doc, deny, msrv)
- [ ] `AGENTS.md`, `CLAUDE.md`, `.deficiencies` from the house templates. `.goals` and `.baseline` are internal: listed in `.gitignore`, never created or committed. `.stage/` is git-ignored too.
- [ ] Update `origin` to the renamed repository URL

### Project documents, adapted from `~/Active/purrdf`
Swap PurRDF/purrdf for RAF/raf and remove purrdf-only content (vendored corpora, Unicode tables, bindings, GTS, Node).
- [ ] `LICENSING.md`: first-party code under `MIT OR Apache-2.0 OR MulanPSL-2.0`, documentation under CC-BY-4.0, the commercial/proprietary reservation, trademarks, CLA, copyright notice
- [ ] `LICENSE-MIT`, `LICENSE-APACHE`, `LICENSE-MULAN` (byte-pinned SHA-256 `eb7a1d713eb919b146787629e22e4c975cb701f529a65d4d7e0fcd417558bf1c`), `LICENSES/CC-BY-4.0.txt`
- [ ] `docs/LICENSING.zh-Hans.md` with names substituted, plus a new `docs/LICENSING-BACKTRANSLATION.md` record for the changed Chinese source
- [ ] `CONTRIBUTING.md`: design constraints rewritten for RAF (normal-form stability, dogfood gates, workplan completion gate); keeps the toolchain/MSRV, semver, conventional-commit, PR-checklist and contribution-licensing sections
- [ ] `CODE_OF_CONDUCT.md`, with the name changed
- [ ] `SECURITY.md`: same process, address and PGP key; attack surface rewritten (untrusted Rust/TOML/YAML parsing, `#[path]` and reference-scan path traversal, spawned cargo/git subprocesses, crates.io supply chain)
- [ ] `.github/ISSUE_TEMPLATE/{bug_report,feature_request,config}.yml`, `PULL_REQUEST_TEMPLATE.md`
- [ ] `CITATION.cff`, `cliff.toml`, `CHANGELOG.md`
- [ ] SPDX headers on every file; `REUSE.toml`; `reuse lint` passes

### Branding
- [ ] `docs/BRAND.md`: name **RAF (Rust Agentic Foundations)**; machine ids are lowercase `raf-*` / `cargo-*`; positioning, tagline, repository description
- [ ] Family system: reuse the `cat-head-core` SVG group verbatim from `purrdf/docs/purrdf-logo.svg`; this repo's service object is **`service-toolbox`** (a toolbox with a wrench), with the four accent colours isolated to that group
- [ ] `docs/raf-logo.svg`, `docs/social-preview.svg`, `docs/social-preview.png` (rendered with `rsvg-convert -w 1280 -h 640`)
- [ ] `README.md` with the logo, what each crate does, and the dev-dependency usage snippets

### Spec-first capability documents (dogfood)
One permanent OKF document per capability in `docs/capabilities/<slug>.md`, with frontmatter `type: Capability`, `id: cap:<slug>`, `title`, `description`, `state`, `bindings`, `depends`, and the fixed sections Purpose / Contract / Usage / Examples / Constraints / Explanation / Verification.
- [ ] M1 census, `state: designed`: `census-normal-form`, `census-ledger`, `census-detectors`, `census-patterns`, `census-rule-api`, `census-debt-ratchet`, `census-reference-scan`, `census-migration`, `census-cli`
- [ ] M1 workplan, `state: designed`: `okf-profile`, `source-index`, `workplan-document-model`, `workplan-reconcile`, `workplan-complete-gate`, `workplan-generated-docs`, `workplan-cli`
- [ ] M2, `state: considered`, each with a `work/changes/<slug>.md` record: `workplan-entity-history`, `workplan-evidence-claims`, `workplan-prose-assessment`, `workplan-two-way-coverage`, `workplan-current-rationale`
- [ ] `docs/architecture/normal-form.md` and `docs/architecture/okf-profile.md`: the current shared decisions
- [ ] Move the content of `INITIAL_PLANNING_PLAN.md` into these documents and remove the file. The essay's rationale lives on in the Explanation sections.

## Acceptance criteria
- [ ] `make check` passes on the empty workspace, and so does CI
- [ ] `reuse lint` is clean
- [ ] Every capability listed above exists with all 7 sections, and with resolved, current-tense text where the state allows
- [ ] No issue numbers in repo docs (the `.baseline` rule); identities are slugs

## Out of scope
Any crate code (see the issues that depend on this one).


## Comments (9)

### paudley — 2026-10-12T05:08:50Z

# Plan — issue #1: Scaffold workspace, project docs and branding, plus spec-first capability docs

- Worktree: `/home/paudley/Active/rust-agentic-foundations/.worktrees/1-scaffold-workspace-project-docs-and`
- Branch: `paudley/1-scaffold-workspace-project-docs-and`
- Base: `origin/main` @ `720262d` (contains only `INITIAL_PLANNING_PLAN.md`)
- Stage directory: `.stage/scaffold-workspace-project-docs-and/`. It is **git-ignored and never committed** (user decision). `.gitignore` carries `/.stage/`.

## Issue summary
Turn the empty repository into the RAF (Rust Agentic Foundations) workspace. That means:
- the toolchain, lints and gates (Makefile, pre-commit hook, CI);
- the house agent files (`AGENTS.md`, `CLAUDE.md`, `.deficiencies`). `.goals` and `.baseline` are internal: git-ignored, never created or committed in the repo (user decision);
- the purrdf-derived legal and community documents, renamed with purrdf-only content removed;
- the RAF brand: the shared `cat-head-core` holding a `service-toolbox`;
- **21 spec-first capability documents** and **2 architecture documents** in the OKF capability format that `raf-workplan` will later enforce;
- the planning essay dissolved into those documents and deleted.

No `raf-*` crate code.

Evidence base:
- `issue.md`;
- `issue-analysis.md`, which traces requirements R1–R44, records the verified environment facts and the ambiguities A1–A19;
- `prior-art-assessment.md`;
- the approved design plan `~/.claude/plans/i-would-two-create-snoopy-sifakis.md`.

## Baseline defaults, standing goals, governing decisions
- The repo has no ADRs or constitution. `.goals` and `.baseline` are the user's internal files: they are git-ignored and never committed (user decision), but their rules still bind the work: hard failures, no deferrals, no issue numbers or development-process references in repo docs, no attribution text, signed commits, hooks never bypassed. The repo enforces the issue-reference and process-wording ban mechanically (the `issue-refs` gate and the docs contract test), so the rule holds without committing `.baseline`.
- **User decisions already made:** `.goals`/`.baseline`/`.stage/` never committed; census code may be ported from any existing copy (including the AGPL-labelled ones) with no notice and no relicensing statement; RAF naming (`raf-*` libs, `cargo-*` bins); brand service object is a toolbox/wrench; licensing, contributing, code of conduct, security and branding docs come from purrdf; a fresh normal form; workplan M1 scope; at most 10 issues.
- **Machine facts:**
  - nightly is the default toolchain;
  - MSRV 1.97.1 runs only through `rust-validate 1.97.1 <cmd>`;
  - `reuse` is not installed, but `uvx reuse` works;
  - a fetching `cargo deny check` is blocked by the machine signed-refs hook, while `cargo deny --offline check` works.

## Prior art and recurrence
- **Templates.** purrdf supplies `.goals`, `.baseline`, `.deficiencies`, `.githooks/pre-commit` (its staged-index snapshot pattern), `clippy.toml`, the CI msrv job pattern, `cliff.toml`, `CITATION.cff`, the issue and PR templates, and every legal and community doc. logit-loom supplies the lint set, `rust-toolchain.toml` and `rustfmt.toml`. lillith_memetics supplies `deny.toml`. gmeow-gts supplies `REUSE.toml`.
- **Gaps in the plan's attribution.** logit-loom lacks most of the files the approved plan attributes to it, so the purrdf and memetics versions are used. logit-loom's CI also never actually runs the MSRV, so purrdf's pinned-and-asserted msrv job is the pattern.
- **Recurrence of the problem RAF addresses:** 7 vendored copies of helper-census, about 54.5k lines, drifted in hash, threshold, ledger dialect and licence. The vendoring did not hold because each copy forked locally, with no publish-and-depend path. Defect-class recurrence is **UNKNOWN**: there are no trailers yet, so this is the first recorded instance.
- **Prior attempts** at a shared Rust tooling workspace or workplan tool: none found locally.

## Decisions taken in this plan (resolving the analysis ambiguities)
| ID | Decision |
|---|---|
| A1 | The root `Cargo.toml` is a **non-virtual root package** `rust-agentic-foundations`: `publish = false`, no dependencies, `src/lib.rs` holding the crate doc, `[lints] workspace = true`, every `[workspace.package]` field inherited, `members = []`. It later hosts the root dogfood tests. No `raf-*` crate is created, so the shared-crates issue is not pre-empted. |
| A2 / K8 | `make check` includes the MSRV build, `$(MSRV_RUN) cargo check --workspace --all-targets --locked` with `MSRV_RUN ?= rust-validate 1.97.1`. In CI the msrv job pins `RUSTUP_TOOLCHAIN=1.97.1` and asserts `rustc --version`. |
| A3 | CI also runs `reuse lint`. |
| A4 | Ship both the root `LICENSE-*` files and `LICENSES/{MIT,Apache-2.0,MulanPSL-2.0,CC-BY-4.0}.txt`. `LICENSES/MulanPSL-2.0.txt` is byte-identical to `LICENSE-MULAN`; a `make check` step verifies both against the pinned digest, so `LICENSING.md` claims only checks that exist. |
| A5 | `.goals` and `.baseline` are **not created**; `.gitignore` lists `.goals`, `.baseline` and `/.stage/` (as purrdf does). `.deficiencies` (tracked, as in purrdf) keeps the notice and marker with no entries. `AGENTS.md` states the issue-reference, process-wording and no-deferral rules itself, since contributors cannot see `.baseline`. |
| A6 / K3 | The zh-Hans doc keeps purrdf's sentences byte for byte apart from the name and the deletion of purrdf-only sentences. A **context-isolated** agent that sees only the Chinese file writes the backtranslation, and the record carries real timestamps, byte count and SHA-256. |
| A7 | The CLA terms stay inside `LICENSING.md` and `CONTRIBUTING.md`, as in purrdf. No `CLA.md`. |
| A8 | **M1 change records are created now as well**: `work/changes/<slug>.md` for each of the 16 M1 capabilities, plus the 5 M2 records. A later issue completes them with the tool. |
| A9 | Architecture pages use `type: Architecture`, `id: arch:<slug>`, `title`, `description`. The type is defined in `docs/architecture/okf-profile.md`. |
| A10 | No hand-written `docs/capabilities/index.md`; it is a generated artifact of the workplan model. |
| A11 | `considered` docs state their Purpose and open questions in plain text; the candidate contract sits in `> [!PROPOSED change:<slug>]` blocks. Change `items` carry `claim` only until completion, and `outcome` is optional until then (stated in okf-profile). `designed` docs state the designed contract plainly, each with a matching change record. Nothing reads as delivered. |
| A12 | `bindings: {code: [], tests: [], examples: []}`. Future symbols are never listed. |
| A13 | SPDX placement and licence, one rule stated once in LICENSING.md and REUSE.toml:<br>- **Every `*.md` file is CC-BY-4.0.** This includes CONTRIBUTING, CODE_OF_CONDUCT and SECURITY, so purrdf's inconsistent split is not copied.<br>- SVG/PNG brand assets, code, config and templates use the code licence `MIT OR Apache-2.0 OR MulanPSL-2.0`.<br>- Frontmatter docs carry SPDX as YAML comments inside the frontmatter. Other Markdown uses HTML comments.<br>- SPDX tags quoted in prose are wrapped in `REUSE-IgnoreStart`/`REUSE-IgnoreEnd`. |
| A14 | `make check` runs `cargo deny --offline check` against the cached advisory DB; CI runs a fetching `cargo deny check`. The machine hook is never bypassed. |
| A15 / adv. 6 | `REUSE ?= uvx reuse@6.2.0` and `MSRV_RUN ?= rust-validate 1.97.1`. Each Makefile target first checks that its tool exists (`command -v $(firstword ...)`). A missing tool is a hard failure that names the tool and an install or CI alternative (`pipx install reuse`; `RUSTUP_TOOLCHAIN=1.97.1`), never a skip. |
| A16 | `SECURITY.md` has a supported-versions table for pre-release `0.x`. |
| A17 | The README describes each crate as **designed, not yet published**, links its capability docs, and puts the snippets under "Planned usage". The README is not included into rustdoc. |
| A18 | The architecture page `okf-profile` holds the profile facts; capability `okf-profile` is the `raf-okf` crate contract and links to it. |
| A19 | Lint set: logit-loom's set plus `unused_qualifications`, and clippy `allow_attributes_without_reason = "deny"`. Every manifest allow has a preceding `#` reason comment. |
| — | A **root dogfood test** `tests/docs_contract.rs` (std-only Rust, in the root package) enforces the document contract in `cargo test`:<br>- closed frontmatter keys per type (Capability, Change, Architecture);<br>- the 7 sections, in order;<br>- `state` values;<br>- every `depends` id resolves, and the `depends` graph is **acyclic**;<br>- every relative Markdown link in `docs/` and `work/` resolves to an existing file (and anchor, for `#section` links to H2s);<br>- every `> [!PROPOSED change:<slug>]` names an existing `work/changes/<slug>.md`;<br>- every capability has its change record (all are designed or considered in this issue);<br>- change `items` outcomes, when present, are `implemented`, `dropped` or `moved:change:<slug>`;<br>- the issue-reference and process-wording bans.<br>Each rule has a negative fixture in the test module. It is real enforcement now; the self-gate issue swaps it for `raf_workplan::Gate`. |
| — | `.gitignore`: `/target`, `/.stage/`, `.goals`, `.baseline`. `Cargo.lock` is committed. |
| — | **`issue-refs` gate** (Makefile target, pre-commit fast gate, and the docs test) scans the whole tracked tree, with no exemptions needed now that `.stage/` is ignored. Pattern set:<br>(1) issue and PR tokens: the analysis's exact `#[0-9]{1,5}\b` (which does not hit hex colour tokens), `/issues/[0-9]+`, `/pull/[0-9]+`, `PR ?#?[0-9]+`, `CHANGE-[0-9]+`;<br>(2) development-process wording from the analysis's R36 second grep: `this issue`, `the plan's`, `Task [0-9]+`, `AC[0-9]+`, `per the plan`, `this branch`.<br>It is implemented as `git grep -InE` over the tracked files and fails with every hit listed. |
| — | Census crates (#2–#5) may port code from any existing helper-census copy, including katamari, sparrow and the gmeow branch, with no notice or licensing statement (user decision). Census capability docs may describe the design's origin freely. |
| adv. 8 | `deny.toml` is based on visual-hashing's (SPDX header, `MulanPSL-2.0` in `allow`, `unused-allowed-license = "allow"`), with `allow = ["MIT", "Apache-2.0", "MulanPSL-2.0"]` (more licences are added only when a dependency needs them). |
| adv. 9 | CI pins actions by SHA using purrdf's set (`actions/checkout@<sha> # v7.0.0`, the toolchain action at purrdf's SHA). `reuse lint` in CI runs through `astral-sh/setup-uv` at a pinned SHA, then `uvx reuse lint`, the same command as `make check`. |
| adv. 10 | `cliff.toml` uses `tag_pattern = "v[0-9]*"` (one workspace version). CONTRIBUTING's versioning section records it. |
| adv. 11 | `docs/architecture/okf-profile.md` also records:<br>(a) the state-to-body rule: `designed` means a plain designed contract plus an open change record, `considered` means the candidate contract in PROPOSED blocks, `delivered` means no PROPOSED blocks and no open change;<br>(b) `id: cap:<slug>` coexists with OKF's optional `resource` key (unused for now; `resource` is reserved for an IRI). |
| adv. 12 | `make licenses` (in `check`, and a pre-commit fast gate) verifies the Mulan digest of `LICENSE-MULAN` and `LICENSES/MulanPSL-2.0.txt`, and compares the SHA-256 of `docs/LICENSING.zh-Hans.md` with the hash recorded in `docs/LICENSING-BACKTRANSLATION.md`. |

**Decided:** porting census code from the AGPL-labelled copies needs no notice and no licensing statement (user decision). Nothing is left open.

## Completeness contract
Requirement IDs refer to `issue-analysis.md`, where each has its exact command.

| Requirement | Task | Executable acceptance check (real input → entry point → observed) |
|---|---|---|
| R1 workspace manifest | T1 | `python3 -I` tomllib assertion (resolver 3, edition 2024, MSRV, license, repository); `cargo metadata` shows the root package with those fields and no `warning` on stderr |
| R2 lints | T1 | tomllib assertion on `[workspace.lints]`; awk check for a reason comment on every allow; `git grep` finds no rustflags |
| R3 toolchain | T1 | tomllib: `channel == "nightly"` with clippy+rustfmt; `git grep '#!\[feature'` is empty |
| R4 rustfmt/clippy/deny | T1 | the files exist; `cargo fmt --all --check` rc 0; `clippy.toml` has no msrv or `1.97.1`; `cargo deny --offline check` reports ok |
| R40 `.gitignore`, R41 `Cargo.lock`, R42 minimal root stub | T1 | `git ls-files Cargo.lock`; the root package has 0 dependencies (`cargo metadata`); no `crates/` dir |
| R10 LICENSING | T2 | section-heading grep; contact, CC-BY-4.0 and CLA greps; purrdf-only terms grep is empty |
| R11 / R11b licence texts | T2 | `sha256sum LICENSE-MULAN LICENSES/MulanPSL-2.0.txt` both give `eb7a1d71…bf1c`; CC-BY digest `d557539d…3744`; `cmp` against purrdf; `make check` digest step |
| R12 zh-Hans | T2 | `diff <(sed 's/PurRDF/RAF/g; s/purrdf/raf/g' ~/Active/purrdf/docs/LICENSING.zh-Hans.md) docs/LICENSING.zh-Hans.md \| grep '^>'` prints only SPDX and link lines (deletions only, no new Chinese sentences); the R12 purrdf-only-term grep prints nothing |
| R13 backtranslation | T2 | the record's SHA-256 and byte count equal `sha256sum`/`wc -c` of `docs/LICENSING.zh-Hans.md`; written by an isolated agent |
| R19 / R34 SPDX + REUSE | T1 (headers on T1 files), T2 (REUSE scaffold), every task | `uvx reuse lint` rc 0 (`.stage/` ignored via `.gitignore`) |
| R5 Makefile check | T3 | `make -n check` lists fmt, clippy `-D warnings`, test, doc, deny, reuse, msrv, licenses, issue-refs; with `REUSE=/nonexistent` `make reuse` exits non-zero and names the tool; `make check` rc 0 (final run in T9) |
| R6 pre-commit (staged index) | T3 | executable, `shellcheck` clean; `make hooks` sets `core.hooksPath`; throwaway-clone test: badly formatted staged code is refused, a bad unstaged edit is ignored |
| R7 CI | T3 | `actionlint` rc 0; jobs fmt, clippy, test, doc, deny, msrv, reuse; msrv pinned to 1.97.1 with a version assertion; `gh pr checks` all pass after the PR (T9) |
| R8 house files (amended by user decision) | T4 | `AGENTS.md`, `CLAUDE.md`, `.deficiencies` exist and are tracked; `.deficiencies` has 0 entries; `CLAUDE.md` references `AGENTS.md`; `AGENTS.md` states the issue-reference, process-wording and no-deferral rules; `git check-ignore .goals .baseline .stage/x` matches all three; `git ls-files .goals .baseline .stage` prints nothing |
| R9 origin | T4 (verify only) | `git remote get-url origin` = the renamed URL (already true) |
| R14 CONTRIBUTING | T4 | RAF constraint headings (normal-form stability, dogfood gates, completion gate); the toolchain/MSRV, semver, conventional-commit, PR-checklist and licensing sections are present; documents `rust-validate` and `uvx reuse` |
| R15 CoC | T4 | `diff` with purrdf shows only name changes |
| R16 SECURITY | T4 | PGP block SHA-256 = `6bb700ee…fa62`; contact address; attack-surface section names Rust/TOML/YAML parsing, `#[path]` traversal, subprocesses, crates.io |
| R17 templates | T4 | `yq` parses each `.yml`; no purrdf terms |
| R18 CITATION / cliff / CHANGELOG | T4 | `cffconvert`-style YAML parse via `yq`; `git cliff --unreleased` runs |
| R20 / R43 no purrdf remnants, brand casing | T4–T8 | `git grep -nil purrdf \| sort` equals exactly the allowlist recorded in `validation.md` (`docs/BRAND.md` for the family source, plus census capability docs whose Explanation names the origin); any other file is a defect. `git grep -nwE 'Raf\|rAF' -- '*.md'` prints nothing |
| R21 BRAND | T5 | headings: Name, Positioning, Tagline, Repository description, Family system, Colour tokens, Logo assets |
| R22 cat-head-core verbatim + service-toolbox isolation | T5 | awk extraction of `cat-head-core` from `docs/raf-logo.svg` SHA-256 = `4ec35cf6…a97fe`; every accent colour occurs only inside `service-toolbox`; `xmllint --noout` rc 0 |
| R23 logo / social preview | T5 | `rsvg-convert -w 1280 -h 640 docs/social-preview.svg` output equals the committed PNG (local); PNG is 1280×640 |
| R24 README | T5 | logo reference; one entry per planned crate with "designed" status and capability links; "Planned usage" snippets |
| R25 frontmatter schema | T6–T8 | `cargo test --test docs_contract` (closed keys per type) |
| R26 seven sections in order | T6–T8 | same test |
| R27 census caps (9, designed) | T6 | same test plus a slug-list loop: `for s in $CAPS_M1C; do test -f docs/capabilities/$s.md; done` and a `state: designed` grep |
| R28 workplan caps (7, designed) | T7 | same, `$CAPS_M1W` |
| R29 M2 caps (5, considered) + change records | T8 | same, `$CAPS_M2`; `work/changes/$s.md` exists with `type: Change` |
| A8 M1 change records | T6–T7 | `work/changes/<slug>.md` exists for each of the 16 M1 slugs |
| R30 architecture docs | T6 (normal-form), T7 (okf-profile) | files exist with `type: Architecture`; `cargo test --test docs_contract` validates their keys and resolves every relative link and anchor in `docs/` and `work/` |
| R31 essay migrated and removed | T8 | `test ! -e INITIAL_PLANNING_PLAN.md`; the key-phrase greps from the analysis map each essay section to its destination Explanation section |
| R32 depends integrity | T6–T8 | the docs_contract test resolves every `depends` id and rejects cycles (negative fixtures: a dangling id and a 2-cycle) |
| R35 current-tense, all sections | T6–T8 | the docs_contract test passes (sections non-empty); `git grep -nE '\b(TBD\|TODO\|FIXME\|XXX)\b' -- docs work` prints nothing; `git grep -nwE 'will\|would\|shall\|should' -- docs/capabilities` lists candidates, and the task review records each surviving hit with its justification (normative contract language, inside a PROPOSED block, or an M2 doc) |
| R36 / R38 no issue numbers or process wording, slug ids | T3 (gate), T6–T8 (test) | `make issue-refs` rc 0 over the tracked tree; negative check: a temp file containing `Per #12` or `this issue`, staged in a throwaway clone, makes `make issue-refs` fail; docs_contract negative fixtures |
| R37 no deferrals | T6–T8 | `git grep -niE 'defer\|follow-?up\|left for future\|future work'` returns only hits recorded in `validation.md` as rules prohibiting deferral; the docs_contract outcome-vocabulary rule passes |
| R39 no attribution | all | machine attribution guards on commit and push; review |
| R44 signed conventional commits | all | normal commit flow under the machine hooks |
| R33 make check + CI pass | T9 | one full `make check` rc 0 (recorded in `validation.md`); `gh pr checks` green |

## Enhancements considered and declined
- **Stub `crates/raf-okf` etc. so the workspace has members:** declined. It pre-empts #2 and creates hollow published names. The root package covers the need.
- **Hand-written capability index:** declined. It would be stale by construction until #7 generates it.
- **`CLA.md`:** declined as unrequested scope. The CLA terms already live in LICENSING and CONTRIBUTING.
- **`markdownlint` / `codespell` gates:** declined. The tools are not installed and the issue doesn't require them; adding a gate here would mean installing tools without being asked.
- **A YAML parser dependency in the docs test:** declined. A std-only line parser for the closed top-level keys is enough until `raf-okf` exists. The visible cost is that nested `bindings` structure is checked only shallowly until then.

## Task 1: Workspace manifest, toolchain, lints, Cargo.lock
Files: `Cargo.toml`, `Cargo.lock`, `src/lib.rs`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, `deny.toml`, `.gitignore` (`/target`, `/.stage/`, `.goals`, `.baseline`), each with its SPDX header.
- **Validation:** checks for R1–R4 and R40–R42; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo doc`; `cargo deny --offline check`; `rust-validate 1.97.1 cargo check --workspace --all-targets --locked`.
- Then task review, commit (`build: scaffold RAF workspace`), push, issue update.

## Task 2: Licensing, licence texts, REUSE
Files: `LICENSING.md`, `LICENSE-{MIT,APACHE,MULAN}`, `LICENSES/*.txt`, `REUSE.toml`, `docs/LICENSING.zh-Hans.md`, `docs/LICENSING-BACKTRANSLATION.md`.
- Commit: `docs(license): add licensing texts and REUSE metadata`.
- The backtranslation comes from a separate isolated agent that is given only the zh file path.
- **Validation:** R10–R13 checks; `uvx reuse lint` rc 0.

## Task 3: Gates — Makefile, pre-commit hook, CI
Files: `Makefile` (`check`, `hooks`, `licenses`, `issue-refs`, `msrv`, `fmt`, `clippy`, `test`, `doc`, `deny`, `reuse`), `.githooks/pre-commit` (staged-index snapshot; fast gates: fmt, reuse, licenses, issue-refs), `.github/workflows/ci.yaml` (adds a `reuse` job and an `issue-refs` step).
- Commit: `ci: add make check, pre-commit hook and CI workflow`.
- **Validation:** R5–R7 checks; the throwaway-clone hook test; `actionlint`; `shellcheck`; `make hooks` run in the worktree, so later commits go through the hook.

## Task 4: House files and community docs
Files: `AGENTS.md`, `CLAUDE.md`, `.deficiencies`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`, `.github/ISSUE_TEMPLATE/{bug_report,feature_request,config}.yml`, `.github/PULL_REQUEST_TEMPLATE.md`, `CITATION.cff`, `cliff.toml`, `CHANGELOG.md`.
- **Validation:** R8, R9 and R14–R18 checks; `uvx reuse lint`; `make issue-refs`.
- Commit: `docs: add agent guide, contributing, conduct, security and GitHub templates`.

## Task 5: Branding and README
Files: `docs/BRAND.md`, `docs/raf-logo.svg`, `docs/social-preview.svg`, `docs/social-preview.png`, `README.md`.
- **Validation:** R21–R24 checks (cat-head-core hash, accent isolation, `xmllint`, PNG re-render compare); `uvx reuse lint`.
- Commit: `docs(brand): add RAF brand, logo, social preview and README`.

## Task 6: Census capability docs, normal-form architecture, docs contract test
Files: the 9 census capability docs, their 9 `work/changes/*.md` records, `docs/architecture/normal-form.md`, and `tests/docs_contract.rs`.
- The test ships with a negative fixture for every rule listed in the decisions table: unknown key, missing section, sections out of order, bad state, dangling `depends`, 2-cycle, broken relative link, broken anchor, orphan PROPOSED marker, missing change record, bad outcome, issue reference, process wording. Each must fail.
- Content source: the approved plan's raf-census design, and the census survey (katamari `crates/katamari-helper-census`, purrdf `crates/helper-census`, purrdf `helpers-ledger.toml` header, sparrow `docs/helper-census.md`).
- Each Explanation section is written here with the essay in hand, so the T8 coverage check needs no second pass.
- **Validation:** `cargo test --test docs_contract` (runs more than zero cases, including the negative fixtures); the R27 loop.
- Commit: `docs(capabilities): add census capability specs and normal-form architecture`.

## Task 7: Workplan M1 capability docs and OKF profile architecture
Files: the 7 workplan capability docs, their 7 change records, and `docs/architecture/okf-profile.md` (defining the Capability, Change and Architecture types, closed keys, SPDX placement, PROPOSED and generated block syntax, and when `outcome` is optional).
- Content source: the approved plan's raf-workplan design, `gmeow-ontology/docs/okf.md`, `gmeow-gts/docs/GTS-OKF.md`, and GROUNDING's coverage duty.
- Explanations are written with the essay in hand. okf-profile also records the state-to-body rule and the `id`/`resource` coexistence.
- **Validation:** docs_contract test; the R28 loop.
- Commit: `docs(capabilities): add workplan capability specs and OKF profile architecture`.

## Task 8: M2 capability docs, change records, essay migration
Files: the 5 M2 capability docs (`considered`, with PROPOSED blocks) and their 5 change records; `INITIAL_PLANNING_PLAN.md` is deleted after the R31 coverage map confirms every essay section has a destination. Any gaps are fixed in the destination doc.
- Essay text that breaks house rules (`CHANGE-17`, "deferred", conversational voice) is rewritten, not copied.
- **Validation:** docs_contract test; the R29 and R31 checks; `make issue-refs`.
- Commit: `docs(capabilities): add M2 workplan capability specs and retire the planning essay`.

## Task 9: Full qualification, PR, plan publication, handoff
- Run one full `make check` in the background and record it in `validation.md`.
- Push, then create the PR with `stagectl pr-create` (body: `Closes #1`, changes, actual validation, residual risks) and post the plan to the PR.
- Watch `gh pr checks` until CI is green.
- Post the least-confident and should-know notes to the issue and the PR, including that GitHub's social preview must be uploaded by hand in repository Settings (there is no API).
- Report the PR, branch, checks, and the Stage 2/3 gates that have not run.

## Plan review dispositions (`reviews/plan-review.md`, pass 1: NEEDS-REVISION)
| Finding | Disposition |
|---|---|
| 1 REQUIRED: tracked `.stage/` collides with the gates | **Addressed by user decision.** `.stage/`, `.goals` and `.baseline` are git-ignored and never committed. The old Task 9 (stage-record commit) is removed. The gates scan the whole tracked tree with no exemptions. |
| 2 REQUIRED: `issue-refs` covers numbers only | Addressed. The gate now runs both pattern sets (issue/PR tokens and process wording). The R36 row has a negative check. |
| 3 REQUIRED: inconsistent document-licence split | Addressed. A13 now states one rule: every `*.md` is CC-BY-4.0; brand assets, code and config use the code licence. |
| 4 REQUIRED: link resolution and cycle detection unmapped | Addressed. The docs_contract test scope adds relative link and anchor resolution, `depends` acyclicity and PROPOSED-orphan checks, each with negative fixtures. The R30 and R32 rows are updated. |
| 5 REQUIRED: AGPL provenance-wording constraint for T6 | **Superseded by user decision**: use the code with no notice and no licensing statement. Census docs may describe the origin freely. |
| 6–14 ADVISORY | Applied: 6 (tool-missing hard fail), 7 (SPDX in T1), 8 (visual-hashing `deny.toml` base), 9 (SHA-pinned actions, uv-based reuse in CI), 10 (`tag_pattern`), 11 (okf-profile state-to-body rule and `id`/`resource`), 12 (zh-Hans fingerprint and Mulan digest gate), 13 (Explanations written in T6/T7), 14 (per-task commit subjects, social-preview upload note). |
| Rubric ground 1 (four "reviewed" cells) | Rewritten as mechanical checks in the R12, R20/R43, R35 and R37 rows. |
| Pass 2 | **PASS**, no open items. Advisories applied: T6 fixture list synced with the test scope; exact `#[0-9]{1,5}\b` regex and `git grep -I`; `uvx reuse@6.2.0` pinned with a `firstword` tool check; `Unicode-3.0` dropped from `deny.toml`; the PR body and issue update state the `.goals`/`.baseline` checklist amendment. |


### paudley — 2026-10-12T05:11:32Z

**Task 1 (workspace scaffold) is committed and pushed:** `a4eacdc build: scaffold RAF workspace`.

- **What it adds:** a non-virtual root package `rust-agentic-foundations` (`publish = false`, no dependencies, members currently empty). Workspace lints (missing_docs, unreachable_pub and unsafe_code denied; clippy all and pedantic; every allow has a reason comment). Floating nightly toolchain, MSRV **1.98.1**, rustfmt, clippy and cargo-deny configs. A `.gitignore` covering `/target`, `/.stage/`, `.goals` and `.baseline`.
- **Checks run, all exit 0:** tomllib manifest assertions; `cargo metadata` with 0 warnings; `cargo fmt --check`; `cargo clippy -D warnings --locked`; `cargo test`; `cargo doc`; `cargo deny --offline check`; `rust-validate 1.98.1 cargo check --workspace --all-targets --locked`.
- **Review:** the independent task review found one problem: the word "rustflags" appeared in a manifest comment and would trip the no-rustflags grep. I reworded the comment and the recheck passed.
- **Checklist change (owner decision):** `.goals` and `.baseline` are internal files. They are git-ignored and never created or committed; the house-file checklist item is amended accordingly.


### paudley — 2026-10-12T05:15:25Z

**Task 2 (licensing) is committed and pushed:** `fc6f531 docs(license): add licensing texts and REUSE metadata`.

- **What it adds:**
  - Root `LICENSE-MIT`, `LICENSE-APACHE` and `LICENSE-MULAN`, plus REUSE `LICENSES/` copies of all three and CC-BY-4.0. `LICENSE-MULAN` matches its pinned SHA-256, `eb7a1d71…bf1c`.
  - `REUSE.toml`, stating one rule: every Markdown file is CC-BY-4.0; code, config, templates and brand assets use `MIT OR Apache-2.0 OR MulanPSL-2.0`.
  - `LICENSING.md`, adapted from purrdf with the purrdf-only material removed.
  - A Simplified Chinese explanation, edited by deleting lines only, with no new Chinese written.
  - A backtranslation record produced by an isolated model that read only the Chinese file. The record carries the source's SHA-256, its byte count and the timestamps.
- **Checks:** the licence digests and `cmp` checks match; the Chinese-file diff shows only deletions; the backtranslation fingerprint matches the source file; `uvx reuse@6.2.0 lint` is compliant (12/12 files); no purrdf leftovers.
- **Review:** the independent task review passed with no required findings.


### paudley — 2026-10-12T05:21:14Z

**Task 3 (gates) is committed and pushed:** `b7db728 ci: add make check, pre-commit hook and CI workflow`. This commit was the first to pass through the new pre-commit hook.

- **`make check`:** runs fmt, clippy `-D warnings`, test, doc, `cargo deny --offline`, `uvx reuse@6.2.0 lint`, the MSRV build, licence digests, and the issue-reference gate.
  - The MSRV step runs through `rust-validate 1.98.1` and checks that the compiler really is rustc 1.98.1.
  - The licence-digest step checks the Mulan text and the Chinese-explanation fingerprint.
  - The issue-reference gate catches issue and PR tokens plus process wording, and its patterns are written so they don't match themselves.
  - A missing tool is a hard failure that names the tool.
- **`.githooks/pre-commit`:** runs the fast gates against the staged index, using an index snapshot and `git grep --cached`. Install it with `make hooks`. The machine config hooks keep running alongside it.
- **CI (`.github/workflows/ci.yaml`):** jobs fmt, clippy, test, doc, deny (fetches the advisory database), msrv (1.98.1, with a version assertion), reuse and gates. Every action is pinned to a full SHA.
- **Essay removed:** `INITIAL_PLANNING_PLAN.md` is gone from the tree, because the whole-tree gate rejects its `CHANGE-17` reference. Its content is read from the base commit while the capability docs absorb it.
- **Review:** the review found one wrong pin comment (it called a stable-branch SHA `v1`). I fixed it and also added the MSRV version assertion; the recheck passed.


### paudley — 2026-10-12T05:28:36Z

**Task 4 (house files and community docs) is committed and pushed:** `c7edb76`.

- **What it adds:**
  - `AGENTS.md` (the canonical agent contract) and `CLAUDE.md` (a pointer to it).
  - `.deficiencies`, with no entries.
  - `CONTRIBUTING.md`, with RAF's design constraints (normal-form stability, hard failures, dogfood gates, the workplan completion gate), the toolchain and MSRV policy, versioning, and contribution licensing.
  - `CODE_OF_CONDUCT.md` and `SECURITY.md`. SECURITY keeps purrdf's process and its PGP block byte for byte, with an attack-surface section rewritten for RAF.
  - Issue and PR templates, `CITATION.cff`, `cliff.toml` (tag pattern `v[0-9]*`; strips trailing PR and issue references), and `CHANGELOG.md`.
- **Checks:** PGP block digest; YAML and CFF validation; `git cliff --unreleased`; `make issue-refs`; reuse (27/27 files); purrdf and brand-casing greps.
- **Review:** the review blocked on present-tense claims that dogfood gates already exist. I reworded them as designed (the document-contract test lands with the capability docs), and the recheck passed.


### paudley — 2026-10-12T05:31:57Z

**Task 5 (branding and README) is committed and pushed:** `49eb42b`.

- **What it adds:**
  - `docs/BRAND.md`: name, positioning, tagline, family system, colour tokens and assets.
  - `docs/raf-logo.svg`: the shared `cat-head-core` copied byte for byte (SHA-256 `4ec35cf6…a97fe`), biting a new `service-toolbox` group (a toolbox with a wrench). The accent colours appear only inside that group.
  - `docs/social-preview.svg` and `docs/social-preview.png` (1280×640).
  - `README.md`, marking every crate *designed — not yet published* and showing the usage snippets under **Planned usage**.
- **Checks:** cat-head-core hash, accent isolation, `xmllint`, PNG re-render compare, `reuse lint` (32/32), and `make issue-refs`. I also rendered both images and checked them by eye.
- **Review:** passed with no required findings. On its advice, the README now says "designed" in its opening sentence.
- **Manual step after merge:** upload `docs/social-preview.png` in repository Settings → Social preview. GitHub has no API for this.


### paudley — 2026-10-12T05:46:26Z

**Task 6 (census capability specs) is committed and pushed:** `25e1978`.

- **What it adds:**
  - 9 census capability documents, all `designed`: normal form, ledger, detectors, patterns, rule API, debt ratchet, reference scan, migration and CLI.
  - Their 9 change records in `work/changes/`.
  - `docs/architecture/normal-form.md`, the normative v1 definition.
  - `docs/architecture/okf-profile.md`: closed key sets for Capability, Change and Architecture; the seven sections; the rule mapping each state to its document body; PROPOSED and generated block syntax.
- **Document-contract test:** `tests/docs_contract.rs` is std-only and has 46 tests, including a negative fixture for every rule. It is written and reviewed, and will be committed together with the workplan docs, because README links to those docs and the commit would otherwise fail its own test.
- **Review:** the first review found three contract inconsistencies. I settled them as follows:
  - Debt growth always fails, and shrinkage fails until `debt --write`.
  - `Rule::fixtures()` is required, and the Gate refuses a rule with missing or bad fixtures.
  - `migrate --out` is required.

  Several advisories are also applied. The recheck passed.
- **Process note:** the pre-commit hook correctly rejected a first commit attempt, whose index held an empty placeholder for the test file.


### paudley — 2026-10-12T05:53:08Z

**Task 7 (workplan capability specs) and the document-contract test are committed and pushed:** `9ce066f` and `b499578`.

- **Capability docs:** 7 workplan-side documents, all `designed`: okf-profile, source-index, workplan-document-model, workplan-generated-docs, workplan-reconcile, workplan-complete-gate and workplan-cli. Each has its change record.
- **Essay content:** sections 1–4, 6 and 7 of the planning essay are rewritten in current tense into these documents. That covers the reconcile principle, permanent capability homes, design in the shape of docs, documentation obligations, the graph and the Grounding duty, and Scenarios A/B.
- **Test:** `cargo test --test docs_contract` now passes 46/46 with zero violations against the real tree, so `make check` and CI enforce the document profile from here on.
- **Review:** the review blocked on three consistency fixes. I made all three, and the recheck passed:
  - Binding minimums apply only to `delivered`.
  - A `contract_effect: none` change still needs all five completion conditions.
  - The generated catalogue `index.md` is the one exception allowed in `docs/capabilities`.


### paudley — 2026-10-12T06:06:48Z

## Stage 1 handoff: PR #11

**Qualification**
- `make check` passed locally on `9f93e5d`: fmt, clippy `-D warnings`, 46 document-contract tests, doc, `cargo deny --offline`, reuse lint, MSRV 1.98.1, licence digests and issue-refs. The only later commit (`d6b6572`) touched one Markdown file, and I re-ran the checks it affects.
- **Hosted CI did not run.** GitHub refused to start every job: *"recent account payments have failed or your spending limit needs to be increased."* No CI step has executed, so CI status is **unverified**, not passing. After billing is fixed, re-run the workflow. This will also be the first time the new `ci.yaml` actually runs: its SHA pins, the msrv toolchain override and the uv-based reuse job have only been linted (`actionlint`), never executed.

**What I'm least confident about, and why**
1. **The CI workflow has never executed.** It is the least verified part of the PR, for the reason above. Likely first-run problems are the pinned toolchain action interacting with `rust-toolchain.toml`, and the cost of `cargo install cargo-deny`.
2. **Design details the capability docs add beyond the approved plan.** The writers filled gaps; reviewers checked each against the plan and found none that contradicts it. They are still decisions you haven't explicitly made:
   - the JSON report and debt-file shapes;
   - `debt --write` refusing to record growth;
   - the `--root` / `--ledger` / `--out` CLI options;
   - `Rule::fixtures()` as a required trait method;
   - three extra reconcile obligation kinds;
   - binding minimums for `delivered` capabilities.
3. **The Chinese licensing explanation** is a draft with no legal or native-speaker review. The backtranslation is model-based.

**What you might not know but should**
- **Owner decisions applied that differ from the issue text:** MSRV is 1.98.1; `.goals`, `.baseline` and `.stage/` are never committed; census code may be ported from any copy without notice.
- **The planning essay is gone from the tree** because the issue-reference gate rejects its `CHANGE-17` example. Its content lives in the 21 capability docs, and the original remains at the base commit.
- **The docs say the document-contract test enforces the profile "today".** That is true once this PR merges. The census and workplan gates are described as designed, not as existing.
- **Local `cargo deny` runs offline.** The machine signing guard blocks fetching the advisory DB, so only CI checks advisories freshly, and CI is currently blocked by billing.
- **Manual step after merge:** upload `docs/social-preview.png` in repository Settings → Social preview.
- **Stage gates not yet run:** Stage 2 (gap analysis) and Stage 3 (merge assessment and squash merge).


