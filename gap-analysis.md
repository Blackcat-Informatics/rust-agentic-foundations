# Gap analysis: issue 1 / PR 11

- Branch `paudley/1-scaffold-workspace-project-docs-and`, head `d6b6572`; base `origin/main` @ `720262d` (holds only `INITIAL_PLANNING_PLAN.md`). 10 commits, 86 files.
- Read-only on repo source. The only file written is this report.

## Reviewed scope

**Inputs read in full:**
- `issue.md` (body, 4 acceptance criteria, 9 author comments), `issue-analysis.md` (R1-R44), `plan.md` (including the user decisions: MSRV 1.98.1; `.goals`/`.baseline`/`.stage` internal; census porting freely), `validation.md`, `tasks/T1..T8-review.md`, `reviews/completeness-review.md`, `historical-fit.md`, `branch-under-review.md`, `raw/pr-11*.json`, `AGENTS.md`, `CLAUDE.md`.
- The essay blob `git show origin/main:INITIAL_PLANNING_PLAN.md` (248 lines), mapped section by section.
- The approved design plan `~/.claude/plans/i-would-two-create-snoopy-sifakis.md`, used to adjudicate the "least confident" additions.

**Repo files read in full:**
- Build and gates: `Cargo.toml`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, `deny.toml`, `src/lib.rs`, `.gitignore`, `REUSE.toml`, `Makefile`, `.githooks/pre-commit`, `.github/workflows/ci.yaml`.
- Project docs: `README.md`, `AGENTS.md`, `CONTRIBUTING.md`, `SECURITY.md` (process and attack surface), `LICENSING.md`, `docs/LICENSING.zh-Hans.md`, `docs/LICENSING-BACKTRANSLATION.md`, `docs/BRAND.md`, `CHANGELOG.md`, `CITATION.cff`, `cliff.toml`, `.deficiencies`, the PR template and issue-template `config.yml`.
- Specs: **all 21 capability documents**, both architecture pages, and 5 sampled change records.
- `tests/docs_contract.rs`: the rule, scan and loader regions only. Mutation depth is left to the quality review.

**Commands run on head d6b6572 (all executed now):**

| Command | Result |
|---|---|
| `cargo test --test docs_contract --locked` | 46 passed, 0 failed |
| `make issue-refs licenses` | rc 0 |
| `uvx reuse@6.2.0 lint` | compliant, 77/77 files |
| Slug/state/keys/id loop, fence-aware section-order loop over all 21 docs | silent |
| `ls docs/capabilities work/changes` | 21 + 21 |
| cat-head-core awk hash | `4ec35cf6…a97fe` (equal to purrdf) |
| Accent-isolation script | `[]` for `docs/raf-logo.svg` |
| `rsvg-convert -w 1280 -h 640` re-render | `cmp`-identical; PNG is 1280x640 |
| `xmllint --noout` (both SVGs) | ok |
| SECURITY PGP block | `6bb700ee…fa62` |
| Mulan, CC-BY and Apache digests / `cmp` | match |
| zh-Hans diff against sed-renamed purrdf | deletions only, plus the ® on line 1 |
| R36 token and process-wording greps; R37 deferral grep; attribution grep | only rule statements, pattern definitions and Cargo's `@generated` header |
| `git log %G?` | all 10 commits signed and conventional |
| `git check-ignore` / `git ls-files .goals .baseline .stage` | ignored / untracked |
| `cargo metadata` | 0 warnings |
| `cargo deny --help` | `--locked` is a valid flag |
| `issue-refs` regex probe | hits `fill="#000"` and `&#38;` |
| `git log` on purrdf `origin/main` | the house squash keeps the branch as a second parent (`dd79ef77e` has two parents) |

**Not run (per brief):** full `make check`. The recorded full run (rc 0 on `9f93e5d`) stands, because `d6b6572` changed one Markdown file and every check that file affects was re-run above.

**Not duplicated:** the CI security deep dive (security review) and docs_contract mutation testing (quality review). External reviewer feedback: none (0 reviews, 0 inline comments, 0 threads).

## Requirement-by-requirement table

| # | Requirement (issue checklist / AC) | Evidence | Status |
|---|---|---|---|
| 1 | `Cargo.toml` workspace: edition 2024, resolver 3, `rust-version`, `[workspace.package]` | `Cargo.toml:18-28`. MSRV is `1.98.1` by owner decision. `cargo metadata`: 1 package, 0 warnings | COMPLETE (amended) |
| 2 | `[workspace.lints]` logit-loom set, reasons on allows, no rustflags | `Cargo.toml:30-50` equals logit-loom plus `allow_attributes_without_reason`. Each allow has a comment above it. No `.cargo/`; no rustflags hit | COMPLETE |
| 3 | `rust-toolchain.toml` floating nightly with clippy and rustfmt; no `#![feature]` | `rust-toolchain.toml:4-7`; `git grep '#!\[feature'` empty | COMPLETE |
| 4 | `rustfmt.toml`, `clippy.toml`, `deny.toml` | Present; `clippy.toml` has no msrv | COMPLETE |
| 5 | Makefile `check` (fmt, clippy `-D warnings`, test, doc, deny, `reuse lint`) | `Makefile:77-86` adds msrv, licenses and issue-refs. Missing tools hard-fail (`Makefile:40-48`) | COMPLETE |
| 6 | `.githooks/pre-commit` on the staged index, plus `make hooks` | `checkout-index` snapshot (`.githooks/pre-commit:16-18`) and `git grep --cached`; mode 100755; `core.hooksPath=.githooks` | COMPLETE |
| 7 | `ci.yaml` (fmt, clippy, test, doc, deny, msrv) | All six jobs plus reuse and gates (`ci.yaml:17-144`). Never executed: see G2 | COMPLETE (file); see G2 |
| 8 | `AGENTS.md`, `CLAUDE.md`, `.deficiencies`; `.goals`/`.baseline`/`.stage` ignored and never committed | All tracked; `.deficiencies` has 0 entries; `.gitignore:4-7`; `git ls-files` prints nothing | COMPLETE (amended) |
| 9 | `origin` is the renamed URL | `ssh://git@ssh.github.com:443/Blackcat-Informatics/rust-agentic-foundations.git` | COMPLETE |
| 10 | `LICENSING.md` | All sections present. Digest claim backed by `make licenses`. Wording nit: G14 | COMPLETE |
| 11 | `LICENSE-*` with Mulan pinned, plus `LICENSES/CC-BY-4.0.txt` | Digests match; `LICENSES/` holds 4 texts | COMPLETE |
| 12 | zh-Hans with names substituted, plus a new backtranslation record | Deletions only; fingerprint `01eb7226…` = record, checked by `make licenses`. Remnants: G14 | COMPLETE (G14 nits) |
| 13 | `CONTRIBUTING.md` RAF constraints plus kept sections | Normal-form stability, dogfood gates and completion gate (`:21-44`); development/MSRV, versioning (semver and conventional commits), PR checklist, contribution licensing, conduct | COMPLETE (G7 principle conflict) |
| 14 | `CODE_OF_CONDUCT.md` with the name changed | Diff: SPDX line plus one purrdf-only noun sentence | COMPLETE |
| 15 | `SECURITY.md`: same process, address and PGP; attack surface rewritten | PGP digest match; address ×2. Rust parsing, TOML/YAML/Markdown parsing, `#[path]`/mod/symlink/link traversal, reference-package scanning, cargo/git subprocesses, crates.io (`:40-57`); `0.x` table | COMPLETE |
| 16 | Issue templates and PR template | `yq` parses all three; no purrdf terms | COMPLETE |
| 17 | `CITATION.cff`, `cliff.toml`, `CHANGELOG.md` | Present and valid. Forward defect: G10. Stale changelog: G15 | COMPLETE (G10, G15) |
| 18 | SPDX on every file; `REUSE.toml`; `reuse lint` passes | 77/77 compliant | COMPLETE |
| 19 | `docs/BRAND.md`: name, machine ids, positioning, tagline, repository description | `BRAND.md:6-34` | COMPLETE |
| 20 | `cat-head-core` verbatim; `service-toolbox` holds the four accents | Hash equals purrdf; isolation `[]`; one `service-toolbox` | COMPLETE |
| 21 | `raf-logo.svg`, `social-preview.svg/.png` rendered with `rsvg-convert -w 1280 -h 640` | Re-render identical; 1280x640 | COMPLETE |
| 22 | README with logo, per-crate description and dev-dependency snippets | `README.md:4-60`. All 10 relative links resolve. Snippets match the `Gate` APIs. Version nit: G15 | COMPLETE (G15) |
| 23 | 9 census docs, `designed`, frontmatter plus 7 sections | Loops silent | COMPLETE structurally; content defects G3, G5, G13 |
| 24 | 7 workplan docs, `designed` | Loops silent | COMPLETE structurally; content defects G4, G8, G12 |
| 25 | 5 M2 docs, `considered`, each with `work/changes/<slug>.md` | `state: considered` with PROPOSED blocks; 5 records valid | COMPLETE; record defect G6 |
| 26 | `docs/architecture/normal-form.md` and `okf-profile.md` | `type: Architecture`; normative content present | COMPLETE; section semantics missing (G1) |
| 27 | Move the essay's content into these documents and remove the file | `D INITIAL_PLANNING_PLAN.md`; no reference remains. Sections 1, 2, 5, 6 and 7 are migrated with fidelity. Section 3's per-section definitions are lost; section 4's "examples compiling or running" is lost from completion | **PARTIAL** (G1, G4) |
| AC1 | `make check` passes on the empty workspace, and so does CI | Local rc 0 (`raw/make-check-9f93e5d.log`, plus re-runs at head). Hosted CI never executed | **PARTIAL** (G2) |
| AC2 | `reuse lint` is clean | 77/77 | MET |
| AC3 | Every capability has all 7 sections, with resolved, current-tense text where the state allows | All 7 sections, in order, non-empty, in all 21. No TBD or hedging in designed docs. But designed contracts contradict each other in places | **PARTIAL** (G3, G4) |
| AC4 | No issue numbers in repo docs; identities are slugs | `make issue-refs` rc 0; R36 greps clean; ids equal file stems | MET |
| Std | No deferrals / no attribution / signed conventional commits / hooks not bypassed | Greps clean; all commits `G`; hook ran on every commit from `b7db728` | MET |
| Std | Hard failures | Makefile and test hard-fail. The designed census CLI silently skips custom rules (G3) | MET for the tree; G3 for the spec |

## Gap list

### G1 — CRITICAL — Essay content lost in migration, while the PR records the essay as "fully absorbed"

**Where:**
- Essay `origin/main:INITIAL_PLANNING_PLAN.md:78-84`: the meaning of each of the seven sections.
  - Purpose: "what problem it solves and where it fits".
  - Contract: "what behaviour callers can rely on".
  - Usage: "how to use it".
  - Examples: "representative inputs, outputs, and workflows".
  - Constraints: "limits, prerequisites, and unsupported cases".
  - Explanation: "why it works this way and which tradeoffs matter".
  - Verification: "which checks substantiate which claims".
- Destination candidates:
  - `docs/architecture/okf-profile.md:67-71` lists only the section names.
  - `docs/capabilities/workplan-document-model.md:28-30` defers to that page.
  - `AGENTS.md:161-163` gives names only.

**Evidence:**
- `git grep -n -iE 'what problem|behaviour callers|how to use|representative|tradeoff|trade-off|limits,|substantiate' -- docs work AGENTS.md CONTRIBUTING.md README.md` returns only `workplan-generated-docs.md:40` (a matrix column, not the definition).
- T7-review recorded this ("per-section meaning only implicit") as an advisory that was never applied.
- T8-review, the completeness review and the PR body ("The planning essay is fully absorbed into these documents and removed") all record full coverage.

**Smaller losses, same requirement:**
- Essay `:211` linked the GMEOW Grounding Contract at a pinned commit. RAF names it (`workplan-document-model.md:211-213`, `workplan-two-way-coverage.md:92-94`) with no citation, so a reader cannot find it.
- Essay `:239` ("an accurate capability explanation") is missing from Scenario B's list in `workplan-complete-gate.md:124-131`.
- The section-4 loss of "relevant examples compiling or running" is G4.

**Consequence:**
- The format the repository dogfoods has seven mandatory headings with no defined content.
- The boundary between Contract, Constraints and Explanation is undefined. That boundary is what "outside PROPOSED blocks a document states the current contract only" depends on, and it is what M2 prose assessment would judge against.
- The essay file is deleted, so the tree no longer holds this text.

**Why CRITICAL:**
- Under the brief's rubric this is explicit issue work recorded as complete but not done, accepted by three gates.
- The fix is small.

**Fix:**
1. Add a "Section | Holds" table to `docs/architecture/okf-profile.md` "Body rules", with the seven definitions above in current tense.
2. Link it from `workplan-document-model.md` Contract.
3. Add the pinned GROUNDING.md URL to `workplan-document-model.md` Explanation and `workplan-two-way-coverage.md` Explanation.
4. Add "an accurate capability explanation" to Scenario B.
5. Correct the PR-body claim only if anything is left out.

**Falsifiable check:**
- `git grep -n -iE 'callers can rely|prerequisites|substantiate' -- docs/architecture/okf-profile.md` returns at least 3 lines.
- `git grep -n 'GROUNDING.md' -- docs` returns at least 1 line.
- `grep -n 'accurate capability explanation' docs/capabilities/workplan-complete-gate.md` returns 1 line.
- `cargo test --test docs_contract --locked` passes 46/46, and `make issue-refs` returns rc 0.

### G2 — HIGH — Acceptance criterion "and so does CI" is unmet: hosted CI has never executed

**Where:** `.github/workflows/ci.yaml` (8 jobs); PR 11 run `38210871728` attempts 1 and 2.

**Evidence:**
- `raw/pr-11-checks.txt`: all 8 jobs FAILURE in 2-3 s with "The job was not started because recent account payments have failed or your spending limit needs to be increased."
- `validation.md`, Stage 2 section: the re-run is still blocked.

**Consequence:**
- Never exercised on a runner:
  - the SHA pins;
  - the msrv job's toolchain env override (`ci.yaml:103-105`);
  - `cargo install cargo-deny`;
  - the uv-based reuse job;
  - `make licenses issue-refs` on a checkout.
- AC1 is half-met.
- External blocker: an organisation billing action. No agent fix exists.

**Fix:**
1. The owner clears billing and re-runs (`gh run rerun 38210871728`).
2. Merge only on green, or with an explicit owner waiver of the CI half of AC1.

**Falsifiable check:**
- `gh pr checks 11` shows 8/8 pass.
- The msrv job log contains `rustc 1.98.1`.
- The test job log shows `46 passed`.

### G3 — HIGH — The census CLI is defined as the Gate, but it cannot register Rules or LedgerExtensions; the documented adoption path fails for dialect ledgers, and custom rule fingerprints go silently unenforced

**Where:**
- `docs/capabilities/census-cli.md`:
  - `:94-95`: "`check` is the same function as `Gate::assert_clean`".
  - `:42-43`: the only options are `--root` and `--ledger`.
  - `:96-97`: `debt --write` is CLI-only.
- `census-ledger.md:72-75`: "With no extension registered, a non-empty `[job.ext]` is an unknown-key error."
- `census-migration.md`:
  - `:38-39`: dialect fields are written to `[job.ext]`.
  - `:55-57`: "After migration the user runs `cargo census check` … and `cargo census debt --write`."
- `census-rule-api.md:51-59`: rules and extensions are registered only on the Rust `Gate`.
- `census-debt-ratchet.md:75-77, 100-101`: only the CLI writes the record, and it refuses while any non-debt finding exists.

**Evidence:** Read together, the designed contract implies three things:
1. For any ledger that carries `[job.ext]`, `cargo census check` exits 2 (invalid ledger), and `debt --write` can never run. Dialect ledgers are exactly the purrdf and decodes ledgers that migration targets.
2. A job that forbids `rule:<custom-id>` makes the CLI pass where the Gate fails. Nothing makes an unregistered `rule:` id an error: the staleness list in `census-ledger.md:85-91` covers only `body` and `shim`.
3. The "same function" claim and the census-cli Verification bullet ("`check` and `Gate::assert_clean` report the same findings") hold only for ledgers that use neither feature.

**Consequence:**
- The adoption workflow (migrate, then check, then debt --write) is impossible for the repositories the census exists to unify.
- The CLI introduces a silent pass in a fail-hard design.
- This is unresolved text in `designed` docs (AC3). The next census issue would implement a contradiction.

**Fix (owner decision; pick one and state it in all four docs):**
- **(a)** Give the Gate the write path (e.g. `Gate::…​.write_debt()`, run from a repository test or xtask). The CLI then refuses, with exit 2 and a message naming the Gate path, any ledger whose `[job.ext]` or `rule:` ids need registered code. `census-migration` Usage then says to run the repository's census test.
- **(b)** Keep the CLI as the writer, and add a way to run it through the repository's registered rules (for example a `cargo census` mode that invokes a repository test target).

**In either case**, an unregistered `rule:<id>` that is not a built-in detector or a pattern is a hard error in both the CLI and the Gate, with a negative fixture.

**Falsifiable check:**
- census-cli Contract states its behaviour for `[job.ext]` and for unregistered `rule:` ids.
- `census-migration.md` Usage no longer sends dialect ledgers to `cargo census check` (or the CLI can handle them).
- census-ledger staleness or validation lists unknown `rule:` ids.
- Each new rule has a Verification bullet.
- `docs_contract` passes.

### G4 — MEDIUM — "An example stops compiling or a criterion changes" is claimed to raise a reconcile obligation, but no designed obligation kind or completion condition covers it

**Where:**
- The claims: `workplan-document-model.md:213-216` (designed, plain text) and `workplan-two-way-coverage.md:102-105`.
- `workplan-reconcile.md:42-49`: the closed kinds are `unresolved-proposal`, `item-without-outcome`, `binding-missing`, `section-untouched`, `generated-stale` and `tense-mismatch`.
- `workplan-complete-gate.md:35-36`: condition 5 runs `bindings.tests` only, and examples run only under `verify` (`:57-61`), which is not a completion condition.
- Essay `:140` lists "relevant examples compiling or running" among what `complete` requires. Essay `:213` makes the same obligation claim.

**Evidence:**
- No obligation kind is raised by a failing bound example or doctest. "Criterion" is not defined anywhere.
- `binding-missing` covers only a referenced API that disappears.

**Consequence:**
- A designed doc promises behaviour that the designed contract does not provide.
- The essay's examples-at-completion requirement is silently narrowed. Scenario B's "actual usage example" is never executed by the gate.

**Fix:** pick one.
- **(a)** Add an obligation kind `example-failing` (a bound example target or doctest of a listed capability fails to build or run; resolved by a passing `verify` run). Make condition 5 "the bound tests and examples pass", and add a refusal fixture.
- **(b)** Reword `document-model.md:213-216` and `two-way-coverage.md:102-105` to claim only what `binding-missing` covers. Record, as an owner decision, that examples are checked by `verify` only.

(a) matches the essay. Either way, define "criterion" or drop the word.

**Falsifiable check:**
- (a): `grep -n 'example' docs/capabilities/workplan-reconcile.md` shows the new kind, and complete-gate condition 5 names examples.
- (b): `git grep -n 'stops compiling' -- docs` no longer attributes it to reconcile.
- `docs_contract` passes.

### G5 — MEDIUM — The `depends` graph omits edges that the documents' own contracts state

**Where and evidence:**
- **Census and the source index.** `source-index.md:20-23` says "The census resolves ledger homes with it". `census-ledger.md:145-146` resolves homes through the source index, and `census-rule-api.md:132-133` says the Gate reads the workspace through it. `AGENTS.md:60-62` repeats this. Yet no census capability lists `cap:source-index` (`census-ledger.md:13`, `census-normal-form.md:13`).
- **The CLI and the Gate.** `census-cli.md:32, 94-95`: `check` runs the patterns, detectors and debt record and equals the Gate. `depends` (`:13`) lacks `cap:census-rule-api`, `cap:census-patterns` and `cap:census-detectors`; none is reachable transitively.
- **The Gate and the full census.** `census-rule-api.md:58-59`: the Gate "runs the census", including the debt record per `census-cli.md:94-95`. `depends` (`:13`) lacks patterns, debt-ratchet and reference-scan.
- **Prose assessment and evidence.** `workplan-prose-assessment.md:40-41, 46-47`: the digest covers depended-on evidence. `depends` (`:13`) lacks `cap:workplan-evidence-claims`.
- **The OKF reader and `Tree`.** `okf-profile.md` (capability) `:108-109` has a caller supply a `raf_source::Tree`, while `depends: []`.

**Consequence:**
- `change impact` lists dependants through `depends` (`workplan-reconcile.md:33-36`). The generated `relationships` tables are derived from it (`workplan-generated-docs.md:41-43`).
- Once the tool runs, a change to `source-index` would not flag any census capability. The graph already misled the blast-radius section of `historical-fit.md` §4.1.
- `docs/architecture/okf-profile.md:56-57` also never defines what a `depends` edge means.

**Fix:**
1. Add `cap:source-index` to `census-ledger` (and to `census-normal-form` if unit discovery uses `raf-source`).
2. Add `cap:census-rule-api` (and `cap:census-patterns`) to `census-cli`.
3. Add patterns, debt-ratchet and reference-scan to `census-rule-api`.
4. Add `cap:workplan-evidence-claims` to `workplan-prose-assessment`.
5. Drop the `raf_source::Tree` mention from the okf-profile capability. That keeps `raf-okf` the root, as the plan's build order expects.
6. Add one value-rule sentence to the architecture page: "`depends` lists the capabilities whose contract this one relies on."

**Falsifiable check:**
- `docs_contract` still passes (acyclic).
- `yq --front-matter=extract .depends docs/capabilities/census-ledger.md` includes `cap:source-index`.
- `git grep -n 'raf_source::Tree' -- docs/capabilities/okf-profile.md` is empty.

### G6 — MEDIUM — All five M2 change records omit the Constraints section from `affects`, although each capability holds a PROPOSED block for that change in Constraints

**Where:**
- `work/changes/workplan-{entity-history,evidence-claims,prose-assessment,two-way-coverage,current-rationale}.md:9-13` list only contract, usage, examples and verification.
- The capability Constraints sections each open with `> [!PROPOSED change:<slug>]`:
  - `workplan-entity-history.md:76-78`;
  - `workplan-evidence-claims.md:79-81`;
  - `workplan-prose-assessment.md:76-78`;
  - `workplan-two-way-coverage.md:81-83`;
  - `workplan-current-rationale.md:83-85`.

**Evidence:**
- A fence-aware awk comparison prints `proposed-in: constraints contract examples usage verification` against `affects: contract examples usage verification` for all five.
- T8-review's "affects lists only PROPOSED sections" is true only as a subset.

**Consequence:**
- `affects` names "the sections that must be reconciled" (`workplan-document-model.md:70-71`). The records under-declare those sections, so `section-untouched` would never be raised for Constraints.
- No docs_contract rule catches the mismatch.

**Fix:**
1. Add `- cap:<slug>#constraints` to each of the five records.
2. Add a docs_contract rule, with a negative fixture: every section that holds a PROPOSED block naming change X appears in X's `affects`.

**Falsifiable check:**
- The awk comparison prints equal sets for all five.
- `cargo test --test docs_contract` shows 47 tests passing, including the new negative fixture.

### G7 — MEDIUM — The two shell gates contradict the repository's own "Rust first" and "every rule has a negative fixture" rules, and the issue-reference rule lives in two diverging implementations (historical-fit D1, D2 and the duplication half of F2)

**Where:**
- The rules: `AGENTS.md:138-140` ("Checks, generators, gates and tests are written in Rust … Shell is limited to thin glue") and `AGENTS.md:146-147`; `CONTRIBUTING.md:42-44`.
- The shell gates: `Makefile:51-62` (`licenses`: digest comparison plus text scraping) and `Makefile:66-75` (`issue-refs`: 11 regexes).
- The Rust copy: `tests/docs_contract.rs:56-63, 378-440` re-implements the token and wording bans, but scans only top-level, `docs/` and `work/` Markdown (`:450-457`).

**Evidence:**
- No committed input drives `make licenses` or `make issue-refs` to failure. Only manual `/tmp` probes are recorded in T3-review.
- The pattern families are hand-copied twice and already differ. The Makefile's `\bPR ?#?[0-9]+` and the Rust `pull_request_number` differ on prefix handling, and the scopes differ: all tracked files versus Markdown only.

**Consequence:**
- Committed guidance an agent follows is false for the repository's own gates.
- Two homes for one rule, in the repository whose purpose is removing duplicate helpers.

**Fix:** pick one.
- **(a)** Move both gates into a Rust test in the root package (for example `tests/repo_gates.rs` reading `git ls-files`, plus an index mode for the hook). Give each a negative fixture, and have the Makefile and the hook invoke it.
- **(b)** Amend `AGENTS.md:138-140` and `CONTRIBUTING.md:44` to name the two shell gates as the stated exception. Add a committed self-test target that feeds each gate a failing input and expects rc ≠ 0.

In both cases, make the docs_contract ban reuse one pattern set.

**Falsifiable check:**
- The `Rust first` text matches the tree.
- A committed test or target fails on a planted `#12` and on an altered `LICENSE-MULAN`.
- `grep -c 'this[ ]issue' Makefile tests/*.rs` shows a single definition site.

### G8 — MEDIUM — `okf-profile.md` claims the test "enforces the profile today", but the bindings rule is unenforced, including the part that is checkable today

**Where:**
- The claim: `docs/architecture/okf-profile.md:30-32`.
- The rule: `:52-55`, "A list never names a symbol that does not exist, so a capability that has nothing built binds nothing".
- The same rule restated: `workplan-document-model.md:100-101` ("`designed` capabilities carry empty lists") and `workplan-complete-gate.md:89-90`.
- The test: `tests/docs_contract.rs:605-623` checks binding keys only.

**Evidence:**
- A `designed` capability with `code: [raf_okf::Nope]` passes the test.
- The PR body discloses "checks binding shape, not whether bound symbols exist", but the committed page does not.

**Consequence:**
- A committed claim is broader than its enforcement.
- The present-day equivalent (designed or considered implies empty lists) needs no `raf-source` and has no fixture. That breaks the every-rule-has-a-fixture principle.

**Fix:**
1. Add a docs_contract rule, with a negative fixture: a `considered` or `designed` capability with any non-empty binding list is a violation.
2. Qualify `okf-profile.md:30-32`: binding resolution waits for the source index.

**Falsifiable check:**
- The new fixture test fails with the rule message on a non-empty `designed` binding.
- `grep -n 'source index' docs/architecture/okf-profile.md` returns at least 1 line.

### G9 — LOW — The issue-reference gates false-positive on short hex colours and XML character references (fail-loud)

**Where:**
- `Makefile:68`, `#[0-9]{1,5}\b`.
- `tests/docs_contract.rs:378-391` (`hash_number`). Its fixture at `:1684-1688` covers only 6-digit and `#1f2` colours.

**Evidence:** a probe in a temp repo hits `fill="#000"` and `&#38;`. No tracked file hits today.

**Consequence:** the first SVG edit using `#000`, or an entity, fails the hook, `make check` and CI with an "issue token" message. Nothing passes silently.

**Fix:**
1. Require a non-`&` prefix and spare 3-, 4-, 6- and 8-digit hex colours, in both implementations (or the one implementation from G7).
2. Add fixtures for `#000` and `&#38;`.

**Falsifiable check:** the probe gives rc 0 for those two lines and still fails `see #12`.

### G10 — LOW — `cliff.toml` lacks per-release dedup, so house squash merges (which keep the branch as a second parent) produce duplicate bullets

This is historical-fit F1, adjudicated down from MEDIUM.

**Where:** `cliff.toml:29`, `:44` (`split_commits = false`) and `:49-54`.

**Evidence:**
- purrdf `dd79ef77e` has two parents, so git-cliff sees both the squash commit and the branch commits.
- purrdf's BREAKING-marker loss came from its own dedup keeping the non-breaking duplicate. RAF has no dedup, so the marker survives on its own bullet. Only duplication remains.
- The `\(\s*\)` cleanup is also absent.

**Consequence:** the release notes repeat entries at the first tag. The output is cosmetic and reviewable.

**Fix:** port purrdf's `rendered_entries` and `breaking_keys` body together, plus the `\(\s*\)` preprocessor.

**Falsifiable check:** in a scratch clone with a two-parent merge whose branch commit repeats the squash subject, `git cliff --unreleased` prints that bullet once, marked BREAKING if either commit has `!`.

### G11 — LOW — CI installs `cargo-deny` unpinned (historical-fit F3; depth deferred to the security review)

**Where:** `.github/workflows/ci.yaml:94-95`.

**Consequence:**
- The CI verdict, or the `deny.toml` schema acceptance, can change with no repository change. Local runs use 0.20.2.
- Every run rebuilds the tool from source.

**Fix:** use `cargo install --locked cargo-deny@0.20.2` (or a SHA-pinned install action), and record the version beside the Makefile `deny` comment.

**Check:** `grep -n 'cargo-deny@' .github/workflows/ci.yaml`.

### G12 — LOW — The delivered-state binding minimum lives outside the normative profile and is worded two ways

**Where:**
- `workplan-document-model.md:103` ("binds at least one code path and one test").
- `workplan-complete-gate.md:52-53` ("resolving bindings, a test").
- The normative state-and-body table in `docs/architecture/okf-profile.md:80-85` has no binding row.
- `workplan-document-model.md:168-169` says the model "adds rules between documents and none inside one", yet this is a single-document rule.

**Consequence:**
- `raf-okf` and `raf-workplan` implementers get two statements of one rule.
- The test's delivered branch (`tests/docs_contract.rs:1083-1099`) has no minimum.

**Fix:**
1. Add "binds ≥1 code path and ≥1 test" to the delivered row of the profile table.
2. Make the complete-gate wording identical.
3. Reword the "none inside one" constraint.

**Check:** `grep -n 'one code path' docs/architecture/okf-profile.md docs/capabilities/workplan-complete-gate.md` returns 2 lines.

### G13 — LOW — Census designed-contract nits

These are small unresolved or contradictory details. Each needs a sentence or two.

- **a. `--locked` versus "not downloaded".** `census-reference-scan.md:44-45` says `cargo metadata` runs with "`--locked` semantics … does not fetch a package that is not already resolved". `:86-87` says a resolved but not downloaded registry package "makes the scan fail". `--locked` still fetches, so the two statements conflict, and the fetch is network access from a gate. Use `--frozen` (locked plus offline) and add a Verification bullet: an undownloaded registry reference fails, naming the package, with no network.
- **b. `--ledger` means two things.** `census-cli.md:42-43` defines a common `--ledger FILE` (the current ledger), while `migrate --ledger <old.toml>` (`:39`, `census-migration.md:27`) uses it for the legacy input. Either name the migrate input separately (owner: the plan fixed `--ledger old.toml`), or state that the common default does not apply to `migrate`.
- **c. "Shipping code only" is unspecified.** `census-detectors.md:51-52` says detectors that read shipping code only "are marked so in their report", but never says which detectors, and has no Verification bullet (an unapplied T6 advisory). Name the detectors in the table and add a bullet.
- **d. Unreferenced patterns.** `census-patterns.md:45-46` says an unreferenced pattern "fails the staleness check". `census-ledger.md:85-91` omits it, and the Verification at `census-patterns.md:135-136` says only "is reported". Make all three say fail.
- **e. "Every key is required".** `census-ledger.md:44-45` says it for `[[job]]`, then `:64` makes `[job.ext]` optional. Say "every key except `[job.ext]`".
- **f. Empty positive fixture set.** `census-rule-api.md:41-42` says "a rule returns at least one of each", but the refusal list at `:61-67` does not refuse an empty positive set. Add "the positive set is empty".
- **g. Normal-form bump.** `census-debt-ratchet.md:62-71` does not say what `debt --write` does with a record whose `normal_form` differs after a version bump, beyond failing hard. The Explanation (`:109-110`) claims the write "cannot admit new duplication", but deleting the file and re-running `--write` re-baselines growth. State the bump path, and state that review of `census-debt.json` creation diffs is the control.

**Check:** each cited line carries the clarified sentence, and `docs_contract` passes.

### G14 — LOW — Licensing text nits

- **a. Provenance records.** `docs/LICENSING.zh-Hans.md:22` keeps "来源记录" (provenance records). RAF has none; it is purrdf's vendored-corpus vocabulary. Deleting it is a deletion-only edit.
- **b. Distribution packages.** `:20` requires distribution-package text to be byte-identical to the pinned digest. English `LICENSING.md` makes no such statement and no check implements it. Crates are not published yet; see `historical-fit.md` §4.3. Either add the matching English statement plus a packaging check when crates arrive, or delete the sentence.
- **c. Re-run the backtranslation after a or b.** Any zh edit needs a fresh isolated backtranslation and a new record fingerprint, which `make licenses` enforces.
- **d. Ambiguity count.** `docs/LICENSING-BACKTRANSLATION.md:34-41` says the translator "flagged eight possible ambiguities", but lists six. Fix the count or list all eight.
- **e. Documentation contributions.** `LICENSING.md:67-70` ("any contribution … shall be licensed as above (MIT OR Apache-2.0 OR MulanPSL-2.0)") and `:103-104` do not carve out documentation. `LICENSING.md:37-39` and `CONTRIBUTING.md:115-116` put Markdown under CC-BY-4.0. Add the documentation sentence to `LICENSING.md` Contributions, and qualify `:67-70` as code.

**Check:**
- `make licenses` rc 0 after the update.
- `grep -n '来源记录' docs/LICENSING.zh-Hans.md` is empty, if deleted.
- `grep -n 'CC-BY' LICENSING.md` shows the contributions sentence.

### G15 — LOW — Project-document consistency nits

- **a. Dependency version.** `README.md:40` has `raf-census = "0.1"`, but `docs/capabilities/census-rule-api.md:80` has `raf-census = "1"`. One workspace version is `0.1.0`, so use `"0.1"`.
- **b. Stale changelog.** `CHANGELOG.md:9-26` `[Unreleased]` stops at the T4 content. It omits the brand, logo and README; the 21 capability specs and 2 architecture pages; and the document-contract test. Add bullets for them.
- **c. SPDX comment shape.** `AGENTS.md:123-125` says Markdown opens with "the two HTML-comment SPDX lines … exactly as the existing documents do". `.github/PULL_REQUEST_TEMPLATE.md:1-4` uses one multi-line comment. Allow both forms in the text, or split the template comment.
- **d. Enforcement wording.** `AGENTS.md:155-156` says the format is enforced by reviewers "until" `raf-workplan` exists. `AGENTS.md:40-43` and `okf-profile.md:30` say the document-contract test enforces it now. Name the test in §6.

**Check:** a grep for each cited line shows the corrected text, and `make issue-refs` returns rc 0.

### G16 — LOW — CI and hook hardening carried from historical-fit (confirmed as low-cost)

- **F4.** No `concurrency` group in `ci.yaml`.
- **F5.** No `.githooks/pre-merge-commit` exec'ing `pre-commit`. `make hooks` sets `core.hooksPath` (`Makefile:88-89`), so no-conflict merges skip the fast gates, although CI still covers them.
- **F7.** `AGENTS.md:202-203` and `CONTRIBUTING.md:72-73` state that `CITATION.cff` and `[workspace.package]` versions agree, and nothing checks it. Both are `0.1.0` today. This is a rule with no fixture.
- **F9.** No update path (dependabot) for the SHA-pinned actions.

**Check:**
- `grep -n concurrency .github/workflows/ci.yaml`.
- `test -x .githooks/pre-merge-commit`.
- A version-agreement check with a negative fixture.
- `test -f .github/dependabot.yml`.

## Adjudications

### Historical-fit findings

| ID | Historical severity | Adjudication |
|---|---|---|
| F1 | MEDIUM | **LOW gap (G10).** Duplication is real. The BREAKING-loss mechanism needs purrdf's dedup, which RAF's template lacks. |
| F2 | MEDIUM | **Split.** False positives are a LOW gap (G9): fail-loud, zero current hits. The two-implementation drift is part of G7 (MEDIUM). |
| F3 | MEDIUM | **LOW gap (G11).** Reproducibility and supply-chain hygiene; the security review owns the depth. |
| D1 | MEDIUM | **MEDIUM gap, confirmed (G7).** D2 is folded in. |
| F4, F5, F7, F9 | LOW | LOW gap (G16). |
| F6 | LOW | **Not a gap.** The plan fixed the target name; a `fmt-fix` target is optional polish. |
| F8 | LOW | **Not a gap now.** Zero dependencies, so deny passes. Forward note: the first crate pull request will need a `deny.toml` skip and allow review. |
| D3 | LOW | **Not a gap.** The MSRV build is the `#![feature]` guard. |

### "Least confident" design additions versus the essay (and the approved plan)

| Addition | Contradicts the essay? | Notes |
|---|---|---|
| JSON report and debt-file shapes | No | The essay is silent; the plan names "baseline and index JSON". |
| `debt --write` refuses growth | No | Consistent with the plan's "never carries slack". Adoption and bump paths: G3, G13g. |
| `--root` / `--ledger` / `--out` | No | `--ledger` collides with the plan's `migrate --ledger old.toml` (G13b). |
| `Rule::fixtures()` required | No | Strengthens the plan's "a test enforces this". Empty-positive nit: G13f. |
| Three extra obligation kinds (`item-without-outcome`, `generated-stale`, `tense-mismatch`) | No | They implement essay §4 (outcomes resolved, indices synchronized) and §6 (no future-tense prose read as available). They make complete conditions 2-4 redundant with condition 1: redundant, not contradictory. |
| Delivered binding minimums | No | They implement the essay's "verification links" and "feature-to-test traceability". Layering nit: G12. |

## Explicit "no gap" confirmations

- **Workspace and toolchain:** manifest, lints, toolchain, rustfmt, clippy and deny files. No rustflags and no `#![feature]`. `Cargo.lock` committed. The root package is a documented, unpublished stub with 0 dependencies, and no `raf-*` crate pre-empts the shared-crates issue.
- **Gate composition and parity:** `make check` covers every issue-required step plus msrv, licenses and issue-refs. The hook runs fmt, reuse, licenses and issue-refs on the index snapshot. CI covers the issue's six jobs plus reuse and gates, with the same commands; the only documented asymmetry is fetching versus offline deny (plan A14). A missing tool is a hard failure everywhere.
- **Licence artefacts:** digests are pinned and verified by `make licenses`, which is exactly what `LICENSING.md:62-65` claims. REUSE is 77/77, and every `*.md` is CC-BY-4.0 per A13.
- **Brand:** `cat-head-core` is verbatim. The accents are isolated to `service-toolbox` in the logo. The social preview follows purrdf's class-based card pattern. The PNG is reproducible and 1280x640.
- **SECURITY:** the PGP block, address and process are preserved. All five issue-named attack-surface items are present. The supported-versions table covers `0.x`.
- **CONTRIBUTING:** every required section is present (design constraints, development/MSRV, semver, conventional commits, PR checklist, contribution licensing, conduct).
- **Capability inventory:** all 21 slugs exist with the correct state (16 `designed`, 5 `considered`) and closed keys. `id` equals `cap:<file stem>`. Bindings are empty, and the 7 sections are in order and non-empty. The `depends` graph is acyclic and every id resolves (missing edges: G5). 21 change records have valid keys and claims only; outcomes are optional before completion (A11).
- **Designed docs:** no TBD, open question or hedging in any of the 16 (all read in full). Every surviving will/would/should is a conditional or normative statement. Nothing claims delivery. README, AGENTS and SECURITY mark every crate designed and not yet published.
- **Essay migration, sections 1, 2, 5, 6 and 7:**
  - §1: amendments, reconcile and "normal links remain useful".
  - §2: the permanent home, the five-row lifecycle including "being revised", the roadmap, and the branch diff.
  - §5: the full rationale-versus-history table, architecture pages and entity history.
  - §6: all six directions, "planned stays planned", one complete account, Grounding, and maintenance as identifiable work.
  - §7: Scenarios A and B.
  - `CHANGE-17`, "deferred" and the conversational voice are rewritten.
  - The losses are only those in G1 and G4.
- **Standing goals:** no issue or PR numbers or process wording; the `.deficiencies` notice is generic house text. No deferrals: the only hits are rule statements. No attribution text in the tree or in commit messages. All commits are signed and conventional. `.goals`, `.baseline` and `.stage` are ignored and never committed.
- **Reviewer feedback:** none external, so nothing is unaddressed.

## Summary by severity

- **CRITICAL:** G1.
- **HIGH:** G2 (external owner action), G3 (owner design decision).
- **MEDIUM:** G4, G5, G6, G7, G8.
- **LOW:** G9-G16.
