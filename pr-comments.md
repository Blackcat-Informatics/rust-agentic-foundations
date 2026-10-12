# PR #11 — comments and review threads

15 comment(s). **Every one needs a disposition** in
`review-debt.md`: addressed (with the commit hash) or declined (with the
reason posted in reply). Bot comments count.

## 1. paudley — 

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
  - MSRV 1.98.1 runs only through `rust-validate 1.98.1 <cmd>`;
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
| A2 / K8 | `make check` includes the MSRV build, `$(MSRV_RUN) cargo check --workspace --all-targets --locked` with `MSRV_RUN ?= rust-validate 1.98.1`. In CI the msrv job pins `RUSTUP_TOOLCHAIN=1.98.1` and asserts `rustc --version`. |
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
| A15 / adv. 6 | `REUSE ?= uvx reuse@6.2.0` and `MSRV_RUN ?= rust-validate 1.98.1`. Each Makefile target first checks that its tool exists (`command -v $(firstword ...)`). A missing tool is a hard failure that names the tool and an install or CI alternative (`pipx install reuse`; `RUSTUP_TOOLCHAIN=1.98.1`), never a skip. |
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
| R4 rustfmt/clippy/deny | T1 | the files exist; `cargo fmt --all --check` rc 0; `clippy.toml` has no msrv or `1.98.1`; `cargo deny --offline check` reports ok |
| R40 `.gitignore`, R41 `Cargo.lock`, R42 minimal root stub | T1 | `git ls-files Cargo.lock`; the root package has 0 dependencies (`cargo metadata`); no `crates/` dir |
| R10 LICENSING | T2 | section-heading grep; contact, CC-BY-4.0 and CLA greps; purrdf-only terms grep is empty |
| R11 / R11b licence texts | T2 | `sha256sum LICENSE-MULAN LICENSES/MulanPSL-2.0.txt` both give `eb7a1d71…bf1c`; CC-BY digest `d557539d…3744`; `cmp` against purrdf; `make check` digest step |
| R12 zh-Hans | T2 | `diff <(sed 's/PurRDF/RAF/g; s/purrdf/raf/g' ~/Active/purrdf/docs/LICENSING.zh-Hans.md) docs/LICENSING.zh-Hans.md \| grep '^>'` prints only SPDX and link lines (deletions only, no new Chinese sentences); the R12 purrdf-only-term grep prints nothing |
| R13 backtranslation | T2 | the record's SHA-256 and byte count equal `sha256sum`/`wc -c` of `docs/LICENSING.zh-Hans.md`; written by an isolated agent |
| R19 / R34 SPDX + REUSE | T1 (headers on T1 files), T2 (REUSE scaffold), every task | `uvx reuse lint` rc 0 (`.stage/` ignored via `.gitignore`) |
| R5 Makefile check | T3 | `make -n check` lists fmt, clippy `-D warnings`, test, doc, deny, reuse, msrv, licenses, issue-refs; with `REUSE=/nonexistent` `make reuse` exits non-zero and names the tool; `make check` rc 0 (final run in T9) |
| R6 pre-commit (staged index) | T3 | executable, `shellcheck` clean; `make hooks` sets `core.hooksPath`; throwaway-clone test: badly formatted staged code is refused, a bad unstaged edit is ignored |
| R7 CI | T3 | `actionlint` rc 0; jobs fmt, clippy, test, doc, deny, msrv, reuse; msrv pinned to 1.98.1 with a version assertion; `gh pr checks` all pass after the PR (T9) |
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
- **Validation:** checks for R1–R4 and R40–R42; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo doc`; `cargo deny --offline check`; `rust-validate 1.98.1 cargo check --workspace --all-targets --locked`.
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

## User decision (after review)
- MSRV is **1.98.1** (user: "rust 1.98.1 is fine"). It replaces 1.97.1 everywhere in this plan, and in the analysis checks R1, R4 and R7. `rust-validate 1.98.1 rustc --version` gives `rustc 1.98.1 (48a229cea 2026-09-01)`.

## Execution adjustment (before Task 3)
- **The essay is deleted in Task 3, not Task 8.** It contains `CHANGE-17`, so the whole-tree `issue-refs` gate (Task 3) would fail on every commit while the file exists. Tasks 6–8 read the essay from the base commit with `git show 720262d:INITIAL_PLANNING_PLAN.md`. Task 8's R31 coverage map checks that blob against the destination docs. The `REUSE.toml` annotation for the essay is removed in Task 3.
- **Gate patterns are self-non-matching.** Character classes such as `this[ ]issue` keep the gate's own definition, and the docs test's negative fixtures (built with `concat!`), from tripping the gate. No path exclusions are needed.
- **Hook scope.** The pre-commit hook runs `issue-refs` against the index (`git grep --cached`). The Makefile target runs it on the working tree. Setting `core.hooksPath=.githooks` locally is safe: the machine hooks are config-based `hook.*` entries and keep running.

## Execution adjustment (Task 6 split)
- **Task 6a**, `tests/docs_contract.rs` plus `docs/architecture/okf-profile.md`: the profile page and the test that enforces it are written together, so the rules have one source. okf-profile moves from Task 7 to here. Task 7 keeps the `okf-profile` *capability* doc, which describes the `raf-okf` crate contract.
- **Task 6b**, census capability docs, `docs/architecture/normal-form.md`, and the 9 census change records, written against the 6a test.
- Change closed keys: `type, id, title, description, changes, affects, contract_effect, items`. Architecture closed keys: `type, id, title, description`. A Change body requires a non-empty `## Summary`.
- Both parts are reviewed as one task, `T6-review.md`, and committed in two commits.

## 2. paudley — 

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

## 3. paudley — 

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

## 4. paudley — 

### Stage 2 fix R1 (`f711aa2`): workplan and architecture docs

**Gaps closed**
- **G1 (CRITICAL), essay content lost.** The OKF profile now has a "Section | Holds" table with the essay's seven definitions, and the document model links to it. The Grounding Contract is cited at its pinned commit, and Scenario B lists "an accurate capability explanation".
- **G4, examples at completion.** Reconcile has a new `example-failing` obligation. Complete-gate condition 5 now requires the bound tests *and examples* to pass, with a refusal example. "criterion" is removed.
- **G5, workplan `depends` edges.** prose-assessment now depends on evidence-claims. The OKF reader no longer names a `raf_source` type. The profile defines what a `depends` edge means.
- **G6.** All five M2 change records now list `#constraints` in `affects`.
- **G8.** The profile states that binding resolution waits for the source index.
- **G12.** The delivered binding minimum is in the normative table, with identical wording in all three docs.
- **A4.** The reconcile example sentence now matches what the example shows.

**Checks run**
- Every R1 grep and `yq` check in the plan passes.
- `cargo test --test docs_contract --locked` 46/46, so the graph stays acyclic.
- `make issue-refs` rc 0.
- `uvx reuse@6.2.0 lint` 77/77.
- The pre-commit hook passed on the staged snapshot.

**Review:** `G-R1-review`. The first pass BLOCKED on two stale "bound tests" / "criteria" phrases; both were fixed, and the recheck gave PASS.

**Hosted CI:** still blocked by the organisation billing block, so it is unverified.

## 5. paudley — 

### Stage 2 fix R2 (`6f8891e`): census docs

**Gaps closed**
- **G3 (HIGH), user decision "(1) but add an issue for (2)".**
  - `Gate::write_debt` is the write path for every ledger. It is called from a binary or xtask, never from a test.
  - The CLI handles ledgers that need no registered code. For a non-empty `[job.ext]` or an unresolvable `rule:` id it exits 2, naming the Gate path.
  - An unknown `rule:` id is a hard validation error in both the CLI and the Gate.
  - Migration sends dialect ledgers to the repository's census test and xtask.
  - The CLI mode that runs repository rules is tracked separately as an enhancement (issue 12).
- **G5, census `depends` edges.** ledger → source-index; cli → rule-api; rule-api → patterns, debt-ratchet and reference-scan. census-normal-form gets no edge, because it has no source-index usage.
- **G13a–g.**
  - a: `--frozen` replaces `--locked`.
  - b: `migrate --ledger` is separate from the common default.
  - c: shipping-only detectors are named, checked against the purrdf helper-census source: 9 shipping, 5 all.
  - d: an unreferenced pattern fails staleness.
  - e: `[job.ext]` is the only optional key.
  - f: an empty positive fixture set is refused.
  - g: the `normal_form` bump path is documented, with diff review of the record as the control.
- **G15a.** `raf-census = "0.1"`.

**Checks run**
- Every R2 grep and `yq` check in the plan passes.
- docs_contract 46/46, so the graph stays acyclic.
- `make issue-refs` rc 0.
- reuse 77/77.
- The pre-commit hook passed.

**Review:** `G-R2-review`. The first pass BLOCKED because the shipping-only detector list was inverted against the purrdf source. It was corrected with a source citation per detector, and the recheck gave PASS.

## 6. paudley — 

### Stage 2 fix R3 (`6120029`): trust model, containment, subprocess and parser contracts

**User decisions applied (verbatim)**
- "Workspace = build input (Recommended)".
- "Fail-closed, honest (Recommended)", for parser stack overflow.

**Gaps closed (security review M1, M2, L3, L4)**
- **Trust model.** SECURITY.md states that the checked workspace is trusted build input. RAF executes its cargo configuration exactly as `cargo build` and `cargo test` do.
- **M1, path containment.**
  - Every path is canonicalised and must lie inside the workspace root, or inside the reference package directory from `cargo metadata`.
  - Symlinks that escape and non-regular files are refused, and the file type is checked on the opened handle.
  - Fixtures cover symlinked files and directories, `link/..`, loops, FIFOs and manifest target paths.
- **M2, subprocesses.**
  - Every spawn lists its exact argv: no shell, `--frozen`, `--no-deps` for root discovery, and `$CARGO` or else `cargo` from PATH.
  - Document-derived values use `--`, single-argument embedding, or validation plus a pattern.
  - The complete gate requires proof that each bound test ran, since `--exact` alone passes on zero tests. Doc tests are listed and matched strictly, and `no_run`/`compile_fail` bindings are refused.
  - Git: minimum 2.30, full-ref default-branch discovery, and output caps.
- **L4, parser bounds.**
  - A configurable 8 MiB per-file cap.
  - Nesting budgets checked before `syn`, a 64 MiB parse stack and a fuzz target.
  - YAML aliases are refused, and ledger and metadata inputs are bounded.
  - A residual overflow aborts the run, so the gate fails and never passes.
- **L3.** The PGP fingerprint `52E5 4B13 1C29 7D1E AB14 3BAA 0C54 057C 8F9A 0090` and the key expiry are published. The user-ID mismatch is noted.

**Review:** `G-R3-review` by the security reviewer. It took 4 passes; the first three BLOCKED, each finding confirmed by experiment:
- syn overflow shapes;
- libtest's zero-test pass;
- rustdoc's whitespace splitting;
- the `--end-of-options` Git version;
- a fixture that contradicted the budgets.

Recheck 3 gave PASS.

**Checks:** R3 plan checks; docs_contract 46/46; `make issue-refs` rc 0; reuse 77/77; hook passed.

**Owner action, outside the repo:** the PGP key expires on 2026-12-24 and needs renewal.

## 7. paudley — 

### Stage 2 fix R4 (`3e20c61`): licensing consistency and fresh backtranslation

**Gaps closed (G14a–e, A3)**
- **zh-Hans.** Deleted the purrdf-only "来源记录" phrase (G14a) and the distribution-package byte-identity sentence (G14b). Restored the third-party-licence statement as 第三方代码和继承材料保留其原有许可 (A3).
  - **Deletion-only rule.** It now applies at token level: a character-level difflib comparison against the source, with names substituted, shows a single insertion, the ® in the SPDX line.
- **Backtranslation (G14c).** A separate agent produced it; that agent could read only a snapshot of the new Chinese file. The record has new timestamps, 1,712 bytes and SHA-256 `e536de48…3d4b`, and lists all 18 ambiguities the translator flagged (G14d). It keeps the translator's statements separate from the repository check's assessment. The translator reported no fragments.
- **LICENSING.md (G14e).** Documentation contributions are CC-BY-4.0, and the code-licence sentences are qualified as code.

**Checks:** `make licenses` (the fingerprint matches), `make issue-refs`, reuse 77/77, and the hook passed.

**Review:** `G-R4-review`. The first pass BLOCKED: the count was off and the translator reply was not archived. Both are fixed; the recheck gave PASS.

## 8. paudley — 

### Stage 2 fix R6 (`2faa697`): licence and issue-reference gates ported to Rust

**User decision applied (verbatim):** "Port gates to Rust (Recommended)".

**Gaps closed (G7, G9, G16-F7, Q-M5, Q-L1)**
- **`tests/repo_gates.rs`** (std-only, 40 tests):
  - Mulan digests and the zh-Hans fingerprint, checked with a hand-written SHA-256 that is verified against NIST vectors;
  - issue/PR tokens and process wording over **every tracked file**;
  - CITATION.cff version agreement with `[workspace.package]`.
- **Fixtures.** Every rule has a named negative fixture.
- **`tests/support`** is the single definition site for the patterns and the index-derived file list (read under `RAF_GATE_ROOT`). `docs_contract.rs` uses both, so its duplicate patterns and walker are removed.
- **G9.** `fill="#000"`, `stroke='#abc'`, `color: #1f2`, `&#38;` and `#123456` pass; `see #123`, `(#1234)` and `Per #12` fail.
- **New forms.** `issue 12`, `GH-12` and `pull request 11` are now caught, and near-miss prose passes.
- **Binary files.** Only a NUL byte marks a file as binary. Latin-1 text is scanned.
- **Gate wiring.**
  - `make licenses` and `make issue-refs` run the full Rust gate binary, so no zero-match filter is possible. The shell recipes are deleted.
  - The pre-commit hook runs both Rust test binaries with `--no-fail-fast` against the staged snapshot, so it now enforces the document contract too (Q-M5).
  - The CI `gates` job is removed; `cargo test --workspace` covers it.

**Checks run**
- `cargo test`: docs_contract 46/46, repo_gates 40/40.
- `make clippy` (`-D warnings`), `make fmt`, `make msrv` (1.98.1), reuse 79/79, actionlint.
- `git hook run pre-commit` passes, with 0.57 s warm wall time.
- In a scratch clone using real signed commits, the hook refused:
  - a staged `#12`;
  - an altered `LICENSE-MULAN`;
  - a capability missing `## Usage`;
  - a CITATION-only version bump.

**Review:** `G-R6-review`. The first pass BLOCKED because invalid UTF-8 was silently skipped. That is fixed, and the zero-match filter and fail-fast advisories were adopted; the recheck gave PASS.

## 9. paudley — 

### Stage 2 fix R7 (`dba307f`): document contract hardened

**Gaps closed (Q-H1, Q-M1–M4, Q-L2/3/5/6/7, plus the G6/G8/G12 rules and carried items)**
- **Inventory table (Q-H1).** It pins the 21 capabilities with their states, the 2 architecture pages and the change records. Deleting or renaming a capability now fails. Before, deleting a capability together with its change record passed.
- **Resolved text.** Placeholder tokens are refused in any case, spacing or dotted form. A section left empty by comments is refused too.
- **Frontmatter parsing.** These now fail: unterminated quotes, unquoted `: `, a leading indicator character and a bare block indicator. Trailing comments are accepted.
- **PROPOSED markers.** They are detected in any case or spacing, and a near-miss is reported as malformed. Designed docs may not carry them. A section that holds one must appear in its change's `affects` (G6).
- **Bindings.** Designed and considered capabilities bind nothing (G8). Delivered capabilities bind at least 1 code path and at least 1 test (G12). A `moved:change:` target must exist.
- **Links.** Every tracked `*.md` is now checked, including `.github` templates and the README logo. Reference definitions are checked, and so are `href`/`src` attributes, with tag state tracked across lines and reset at blank lines and fences.
- **Placement.** Typed docs must live in their own directory. A generated index may hold only an SPDX header and generated blocks.
- **File reads.** Only regular files are read. FIFOs and symlinks are refused, and a swap after inspection is detected.
- **Negative fixtures.** Every rule now has one: docs_contract went from **46 to 112 tests**.

**Checks run**
- docs_contract 112/112 and repo_gates 40/40.
- `make clippy fmt msrv`.
- reuse 79/79.
- The hook passed.
- In scratch copies, quality-review mutation probes 7, 9, 18, 19, 22, 24, 25, 26, 29, 31, 32 and 34 now each fail with the named rule.

**Review:** `G-R7-review` (quality engineer), 5 passes. The reviewer's evasion probes found:
- a forgeable "generated" index;
- link tag state checked per line;
- prose tag state that persisted across blank lines.

Each was fixed with fixtures. The reviewer confirmed that no original test was weakened, and the final pass is PASS.

## 10. paudley — 

### Stage 2 fix R8 (`cb90d10`): CI pins and merge-commit gates

**Gaps closed (G11 / S-L1, S-L2, G16 F4/F5/F9, S-L5)**
- **Pinned tools.**
  - `cargo install --locked cargo-deny@0.20.2` matches the local version.
  - uv is pinned to `0.13.0`, with `enable-cache: false`.
- **Toolchain action repin.** `dtolnay/rust-toolchain` is repinned to the master head `e2a55d2…`, which is never rebuilt. `git ls-remote` confirms it, and the compare against the input-handling fix `83d0610` reports `ahead`. Every job passes `toolchain:` explicitly.
- **Concurrency group.** Superseded PR runs are cancelled.
- **`.githooks/pre-merge-commit`.** It runs the fast gates. In a scratch clone with real signing, it refused a no-conflict merge that brought in a staged `#12`.
- **Dependabot.** Added for the pinned actions.
- **`$(ROOT)` quoting.** Already quoted in R6; verified with a path that contains a space.

**Checks run**
- `actionlint` rc 0.
- zizmor reports no findings (offline mode).
- `cargo deny --offline check` ok.
- reuse 81/81.
- The hook passed.

**Review:** `G-R8-review` gave PASS.

## 11. paudley — 

### Stage 2 fix R9 (`8ff3859`): changelog dedup

**G10 closed.** `cliff.toml` ports purrdf's per-release dedup (`rendered_entries`/`breaking_keys`) and the `\(\s*\)` cleanup. RAF's tag pattern, parsers and PR-number stripping are unchanged.

**Check.** I built a scratch clone with signed commits and no bypass: a two-parent merge whose branch commit repeats the squash subject.
- With the old config the bullet printed twice.
- With the new config it prints once, and is marked BREAKING when either commit has `!`.
- `git cliff --unreleased` on the real history gives rc 0 and output identical to before.

**Review:** `G-R9-review` gave PASS. One LOW advisory, accepted: when two entries of different types share both scope and subject, the breaking flag over-marks. It errs safe and matches purrdf.

## 12. paudley — 

### Stage 2 fix R5 (`df65ee4`): project-document consistency

**Gaps closed (G15b–d, A1)**
- CHANGELOG `[Unreleased]` now covers everything the tree delivers:
  - brand and README;
  - the 21 capability specs and 2 architecture pages;
  - the document contract;
  - the Rust repository gates;
  - the security contracts;
  - CI pins and hooks;
  - the changelog dedup;
  - the retired essay.
- AGENTS.md §6 names `tests/docs_contract.rs` as the format's enforcement today.
- AGENTS.md and CONTRIBUTING.md describe both hooks and Dependabot.
- The PR template's SPDX header now uses the two-line shape.

**Checks run:** every R5 grep; an SPDX-shape scan of every tracked Markdown file; `cargo test --workspace` (112 + 40); reuse 81/81; `git cliff --unreleased` rc 0; the hook passed.

**Review:** `G-R5-review` gave PASS. Its advisory about the hook description is fixed.

Every remediation group has now landed. The final full `make check` runs next.

## 13. paudley — 

### Stage 2 fix R11 (`45bc98e`, `edf93d5`): exit-audit findings

The first exit-gate completion audit returned FINDINGS-OPEN. Its findings are fixed:

- **F1 (MEDIUM), a gate regression (`45bc98e`).**
  - **Cause:** the Rust issue-reference gate spared a `#` that follows a word character. So `org/repo#12` and `Fixes#12` passed, though the Stage 1 grep caught them.
  - **Fix:** the exemption is removed. An all-digit token after `:` is no longer read as a colour, so `Fixes:#123` fails too. Colours stay legal in attribute values and whenever they contain a hex letter.
  - **Parity:** a parity table against the Stage 1 grep now differs only on the deliberate `fill="#000"` and `&#38;` cases.
  - **Hook:** it refuses a signed scratch commit that adds `Blackcat-Informatics/rust-agentic-foundations#12`.
- **F2–F5 (LOW), spec contradictions (`edf93d5`).**
  - **F2:** the OKF profile now describes links resolved against the tracked file list, and docs_contract refuses a link to a tracked symlink.
  - **F3:** `sole-name` is stated as the exception to the detectors' home-package rule.
  - **F4:** `migrate` also rewrites a current-format ledger written at an older `normal_form`, so the documented bump path exists. The normal-form page names `migrate` as the one reader of an older-version ledger.
  - **F5:** the complete gate states the proof for each binding kind: test, doc test, example.

**Checks:** docs_contract 113, repo_gates 42; `make clippy fmt msrv`; reuse 81/81; each set passes alone; the hook passed on both commits.

**Review:** `G-R11-review` gave PASS. One advisory, the `:` colour context (A1), was adopted.

## 14. paudley — 

### Stage 2 fix R11, round 2 (`80c14d7`): migrate bump input

The exit audit's pass-2 recheck discharged F1–F5 and found one new LOW contradiction, N1, introduced by the F4 fix. It is now fixed.

**N1.** At a `normal_form` bump, `migrate` mapped fingerprints through the baseline report. That report drops single-member groups, so a clean ledger could not be rebuilt.
- `migrate` now reads the **index report** (`cargo census index`) written before the upgrade. The index report lists every unit.
- Only `body:`/`shim:` fingerprints are remapped. `table:`/`rule:` ids, patterns, distinct sets, the reference list and the file-size cap carry over unchanged.
- Fixtures cover each of these.

**Also fixed.**
- docs_contract refuses any tracked symlink.
- The migrate example uses `helpers-ledger.toml`.

**Checks.** docs_contract 114, repo_gates 42; `make clippy msrv`; reuse 81/81; the hook passed.

**Review.** `G-R11-review`, recheck 2: PASS. The reviewer walked the clean-job example through the algorithm.

## 15. paudley — 

## Stage 2 final summary (head `80c14d7`)

**Review path:** full. The change touches CI/supply-chain surface, git hooks, a contract test, and 21 specs that set contracts for later crates.

### Gaps resolved
The reviews were gap analysis, the completion audit, security, quality, historical fit, and the remediation-plan review. Their findings:
- **Gap analysis:** G1–G16, of which 1 CRITICAL, 2 HIGH, 5 MEDIUM and 8 LOW.
- **Security:** M1, M2 and L1–L5.
- **Quality:** H1, M1–M5 and L1–L8.
- **Exit audit:** F1–F5 and N1.

Every finding is **FIXED** or recorded as an evidenced false positive in `remediation-plan.md`. One gap is externally blocked: **G2, hosted CI**.

| Group | Commit(s) | Closes |
|---|---|---|
| R1 workplan docs | `f711aa2` | G1 (essay content), G4, G5, G6, G8, G12 |
| R2 census docs | `6f8891e` | G3 (Gate owns the census path), G5, G13a–g, G15a |
| R3 security specs | `6120029` | trust model, path containment, subprocess argv, test-ran proof, parser bounds, PGP fingerprint |
| R4 licensing | `3e20c61` | G14 (token-level deletions only; fresh isolated backtranslation) |
| R6 Rust gates | `2faa697` | G7, G9, G16-F7, Q-M5 |
| R7 document contract | `dba307f` | Q-H1 (capability inventory), Q-M1–M4, Q-Ls, G6/G8/G12 rules (46 → 112 tests) |
| R8 CI | `cb90d10` | G11, S-L1/L2, G16 |
| R9 cliff | `8ff3859` | G10 |
| R5 project docs | `df65ee4` | G15b–d |
| R11 exit-audit fixes | `45bc98e`, `edf93d5`, `80c14d7` | F1 (gate regression), F2–F5, N1 |

### User decisions this stage (verbatim)
1. "(1) but add an issue for (2)". The Gate owns the census path, and issue 12 covers the CLI running repository rules.
2. "Workspace = build input (Recommended)".
3. "Port gates to Rust (Recommended)".
4. "Fail-closed, honest (Recommended)", for parser stack overflow.

No scope cuts were made, and `.deficiencies` has no entries.

### Reviewer dispositions
- Every group has an independent review in `tasks/G-R*-review.md` ending in PASS.
- Several reviews blocked first, each on a real defect that was then fixed:
  - R3 (3 passes): syn overflow shapes, libtest zero-test pass, rustdoc name splitting, git version.
  - R6: Latin-1 files silently skipped.
  - R7 (4 passes): forgeable index, tag state checked per line.
- The exit audit caught a gate regression introduced during this stage. It is fixed, and a parity table against the Stage 1 gate now differs only on the deliberately spared colour and character-reference cases.

### Validation, kept separate

**Local validation: PASS.**
- `make check` on `80c14d7` returned **rc 0**: fmt; clippy `-D warnings`; docs_contract 114 + repo_gates 42; doc; `deny --offline`; reuse 81/81; MSRV 1.98.1; licenses; issue-refs.
- The pre-commit and pre-merge-commit hooks ran on every commit. In scratch clones with real signed commits, they refused:
  - a planted `#12`, including `org/repo#12`;
  - an altered licence;
  - a capability missing a section;
  - a CITATION version drift;
  - a no-conflict merge carrying a token.

**Exit-gate completion audit: PASS (pass 3).** Every in-scope criterion is demonstrated on `80c14d7`, except the next item.

**Hosted CI: UNVERIFIED (external blocker).** GitHub refused every run on this branch with "recent account payments have failed or your spending limit needs to be increased", most recently run 38229506850 on `80c14d7`. No CI step has ever executed. Only the owner can clear this.

**Unrun live acceptance:** none required. This issue ships no crate code.

### Owner actions outside the repo
- Clear GitHub billing, then re-run CI on the final head, or waive the CI half of AC1 explicitly.
- After merge, upload `docs/social-preview.png` in Settings and set the repository description from `docs/BRAND.md`.
- Renew the security PGP key before 2026-12-24.

### Stage 3 readiness
- **Ready for Stage 3** apart from hosted CI.
- The exit audit and the `80c14d7` qualification can be reused if the head is unchanged and the base delta is assessed.

