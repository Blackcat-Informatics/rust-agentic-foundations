# Completeness review: paudley/1-scaffold-workspace-project-docs-and

Base `origin/main` @ 720262d, head 9f93e5d (9 commits, 86 files, +8184/-248).
Read-only review against `issue.md` (with the owner's amendment: `.goals`/`.baseline` are internal, git-ignored, never committed), `plan.md` (R1–R44, decisions, execution adjustments, MSRV 1.98.1, `.stage/` ignored), `issue-analysis.md` check commands, `validation.md`, and `tasks/T1..T8-review.md`.
`make check` was not run here (it runs separately). Every check below was re-run on the tree for this review.

## Issue checklist

| Issue item | Delivered? | Evidence |
|---|---|---|
| Cargo.toml workspace: edition 2024, resolver 3, rust-version, `[workspace.package]` | Yes (MSRV 1.98.1 by user decision, replacing 1.97.1) | tomllib assert rc 0 (resolver 3, 2024, 1.98.1, authors, license, repository); `cargo metadata`: 1 package, 0 deps, `publish=[]`, 0 `warning` lines |
| `[workspace.lints]`, every allow has a reason, no rustflags | Yes | tomllib: missing_docs/unreachable_pub/unsafe_code deny, clippy all+pedantic warn prio -1, plus unused_qualifications and `allow_attributes_without_reason = "deny"`; awk reason-comment check rc 0; no `.cargo/config.toml`; rustflags grep empty; `[lints] workspace = true` |
| rust-toolchain.toml nightly + clippy/rustfmt; no `#![feature]` | Yes | tomllib assert rc 0; `git grep '#!\[feature'` empty |
| rustfmt.toml, clippy.toml, deny.toml | Yes | all non-empty; no `msrv` in clippy.toml; `cargo fmt --all --check` rc 0; `cargo deny --offline check licenses bans sources`: bans ok, licenses ok, sources ok |
| Makefile `check` (fmt, clippy -D warnings, test, doc, deny, reuse lint) | Yes | `make -n check` runs fmt, clippy `-D warnings`, test, doc (`-Dwarnings`), `deny --offline`, `uvx reuse@6.2.0 lint`, msrv (asserts `rustc 1.98.1`), licenses, issue-refs; `make REUSE=/nonexistent reuse` exits 2 and names the tool |
| `.githooks/pre-commit` (staged index) and `make hooks` | Yes | mode 100755; shellcheck rc 0; `git checkout-index --all --prefix=$snapshot/` plus `git grep --cached`; `core.hooksPath=.githooks`; T3 throwaway-clone index test recorded |
| `.github/workflows/ci.yaml` (fmt, clippy, test, doc, deny, msrv) | Yes | actionlint rc 0; jobs fmt, clippy, test, doc, deny, msrv, reuse, gates; msrv uses `RUSTUP_TOOLCHAIN: 1.98.1` and asserts `^rustc 1\.98\.1 ` |
| AGENTS.md, CLAUDE.md, `.deficiencies` (`.goals`/`.baseline` internal by amendment) | Yes | all three tracked and non-empty; `.deficiencies` has its marker and 0 entries; CLAUDE.md points to AGENTS.md; AGENTS.md states the no-tracker-number, process-wording and no-deferral rules; `git check-ignore` matches `.goals`, `.baseline`, `.stage/x`; `git ls-files .goals .baseline .stage` is empty |
| Update origin | Yes | `ssh://git@ssh.github.com:443/Blackcat-Informatics/rust-agentic-foundations.git` |
| LICENSING.md | Yes | headings Open-source terms (with Documentation CC-BY-4.0), Proprietary / commercial licensing, Trademarks, Contributions (CLA), Copyright notice; contact present; purrdf-only term grep empty |
| LICENSE-MIT/APACHE/MULAN (pinned), LICENSES/CC-BY-4.0.txt | Yes | LICENSE-MULAN and LICENSES/MulanPSL-2.0.txt both `eb7a1d71…bf1c` and identical; CC-BY `d557539d…3744`; Apache identical to purrdf; MIT copyright line names Blackcat; LICENSES/ holds all 4 |
| zh-Hans plus backtranslation record | Yes | lines added against the sed-substituted purrdf file: only the SPDX header; no purrdf terms; record SHA `01eb7226…` and 1,692 bytes match the file; "Complete independent backtranslation" section present |
| CONTRIBUTING.md | Yes | design constraints: normal-form stability, dogfood gates, workplan completion gate; Development (MSRV 1.98.1, rust-validate, uvx reuse), Versioning (semver, conventional commits), "Before you open a pull request", "Licensing of contributions", Conduct |
| CODE_OF_CONDUCT.md | Yes | diff with purrdf: the SPDX licence line (per A13) and one sentence with purrdf-only nouns (library/codecs, now crates/gates) |
| SECURITY.md | Yes | PGP block SHA `6bb700ee…fa62`; address appears twice; attack surface covers Rust source, TOML/YAML/Markdown, `#[path]` and link traversal, reference-package scanning, cargo/git subprocesses, crates.io supply chain; 0.x supported-versions table |
| Issue templates and PR template | Yes | yq parses all 3 `.yml`; purrdf/W3C/python/wasm grep empty; PR checklist covers `make check` and capability reconciliation |
| CITATION.cff, cliff.toml, CHANGELOG.md | Yes | CFF 1.2.0 with title, message and authors; `git cliff --unreleased` rc 0; CHANGELOG non-empty with 0 PurRDF |
| SPDX on every file; REUSE.toml; reuse lint | Yes | `uvx reuse@6.2.0 lint` rc 0, 77/77 files with copyright and licence, compliant with REUSE 3.3 |
| docs/BRAND.md | Yes | "RAF (Rust Agentic Foundations)"; headings Name, Positioning, Tagline, Repository description, Family system, Colour tokens, Logo assets; lowercase `raf-*`/`cargo-*` machine ids; all 7 colour tokens; rsvg command |
| cat-head-core verbatim; service-toolbox holds the accents | Yes | awk extraction SHA `4ec35cf6…a97fe`; one `service-toolbox`, no `service-triple`; accent-isolation script prints `[]`; `<style>` has no accent; toolbox and wrench in title/desc |
| raf-logo.svg, social-preview.svg/.png | Yes | xmllint rc 0; PNG is 1280x640; a fresh `rsvg-convert -w 1280 -h 640` render is `cmp`-identical |
| README.md | Yes | logo; 6 crates marked "designed — not yet published"; "Planned usage" with `[dev-dependencies]` and both `Gate::workspace(...).assert_clean()` snippets (match census-rule-api and workplan-document-model); **all 10 relative links resolve** (6 capability docs, CONTRIBUTING, AGENTS, LICENSING, BRAND) |
| M1 census, 9 docs, `designed` | Yes | R25 loop silent (closed keys, `cap:<slug>`, bindings code/examples/tests all empty); R26 seven sections in order, none empty; state loop silent; R27 spot greps all match (normal_form 1, 20/30/6, BLAKE3 first-8-bytes hex, exit 0/1/2, 9 subcommands, ledger keys, 14 detectors, 6 matchers, Rule trait, debt json, cargo metadata, migrate) |
| M1 workplan, 7 docs, `designed` | Yes | same loops silent; R28 spot greps all match |
| M2, 5 docs `considered` plus change records | Yes | R29 loop silent (Change type and id, `changes`, `contract_effect`, items > 0, affects valid, PROPOSED block present); R29 content (git log --follow/-L, persisted evidence, invalidation, code->docs/tests->claims, conditions to revisit) present |
| docs/architecture/normal-form.md, okf-profile.md | Yes | `type: Architecture`, `arch:<slug>`; normal-form has normal_form 1, thresholds, fingerprint, 4 kinds, version-bump rule, golden tests; okf-profile has keys, values, body, state-and-body, PROPOSED and generated blocks, file rules; linked from census-normal-form, okf-profile and workplan-document-model |
| Essay content moved; file removed | Yes | `D INITIAL_PLANNING_PLAN.md`; no reference to it; every key phrase in the coverage map hits; spot check of essay substance (amendments, permanent home and lifecycle, roadmap, seven sections, design-material mapping, promotion needs reconciliation, obligations and `contract_effect`, generated vs authored prose, the two-place API example, current rationale vs historical deliberation table, shared architecture pages, entity history, six-direction graph, Grounding, documentation maintenance as work, Scenarios A/B); CHANGE-17, "deferred" and the conversational voice are rewritten |
| AC: `make check` passes, and CI | Pending (outside this review) | `make check` runs separately; CI triggers on `pull_request` to main, so there is no run until the PR exists (T9). Every component gate passed on its own here: fmt, deny, reuse, issue-refs, docs_contract (46/46) |
| AC: reuse lint clean | Yes | see above |
| AC: every capability has 7 sections and resolved current-tense text | Yes | R26 silent; no TBD/TODO/FIXME in docs or work; no unresolved markers in designed docs (only prose describing the considered state); will/would in designed docs is conditional or normative ("would merge unrelated tables", "`docs sync` would rewrite nothing"), not future delivery |
| AC: no issue numbers; ids are slugs | Yes | `make issue-refs` rc 0; R36 token regex and process-wording grep empty; "milestone" grep empty (the T8 advisory is fixed); R38 id = file stem for all 42 docs |

## Completeness contract rows (plan.md)

| Row | Delivered? | Evidence |
|---|---|---|
| R1–R4, R40–R42 | Yes | as above; `.gitignore` has `/target`, `/.stage/`, `.goals`, `.baseline`; Cargo.lock tracked; no `crates/` |
| R5, R6, R7 | Yes | as above |
| R8 (amended), R9 | Yes | as above |
| R10–R13 | Yes | as above |
| R14–R18 | Yes | as above |
| R19 / R34 | Yes | reuse 77/77 |
| **R20 / R43** | **Partly** | R43: `git grep -nwE 'Raf\|rAF' -- '*.md'` is empty. R20: `git grep -il purrdf` gives **`docs/BRAND.md`** (line 40, family source: expected) and **`docs/capabilities/census-reference-scan.md`** (lines 54, 59, 75, 78: `reference = ["purrdf-core"]` and `purrdf_core::iri::parse` in **Usage and Examples**). The recorded allowlist (`validation.md`) is only `docs/BRAND.md`, and the plan allows census docs only where the *Explanation* names the origin. The plan row says any other file is a defect. The other purrdf-only-term grep (GTS/PyO3/npm/Node/Unicode-3.0/corpora/wasm-pack) is empty |
| R21–R24 | Yes | as above |
| R25, R26 | Yes | yq loops silent. The naive shell section grep reports a second `## Contract` in workplan-document-model.md (line 141); it sits inside a ```` ```markdown ```` fence, and the fence-aware Rust test accepts it |
| R27, R28, R29, A8 | Yes | 21 capability docs, 21 change records (16 M1 + 5 M2); outcomes all empty (claims only, allowed before completion by A11) |
| R30 | Yes | as above. The contract test resolves links and anchors in docs/ and work/, and a separate sweep of every tracked `*.md` relative link found nothing broken |
| R31 | Yes | as above |
| R32 | Yes | every `depends` target exists; tsort rc 0 (acyclic). The orphan-marker shell loop reports `example-slug` (okf-profile.md:95) and `incremental-parsing` (workplan-document-model.md:53,145); all three are inside code fences, so they are illustrations, not real markers |
| R35 | Yes | as above |
| R36 / R38 | Yes | as above |
| R37 | Yes | deferral grep hits only AGENTS.md:186–188 and CLAUDE.md:20 (rules banning deferral) and AGENTS.md:7 ("CLAUDE.md defers to it", which is not a work deferral); outcome vocabulary clean |
| R39 | Yes | `git log --format=%B origin/main..HEAD` has no co-author trailer, agent name, generated-by credit or vendor address; tree grep hits only Cargo's `@generated` header and Apache licence boilerplate |
| R44 | Yes | all 9 commits signed (`%G?` = G); all subjects conventional (build, docs(license), ci, docs, docs(brand), docs(capabilities) x3, test) |
| R33 | Pending | see the acceptance row above |
| Document-contract test "today" | Yes | CONTRIBUTING.md:33 and docs/architecture/okf-profile.md:30 (and AGENTS.md:42) say the document-contract test enforces the profile today; `tests/docs_contract.rs` exists (1799 lines) and `cargo test --test docs_contract --locked` gives 46 passed, 0 failed, including `repository_documents_satisfy_the_profile` and a negative test for every rule in the plan |
| No stub or hollow requirement | Yes | no empty sections; the smallest prose sections are Usage/Examples whose substance is fenced code; every change record has a non-empty Summary and claims; root package is a documented `src/lib.rs` with no hollow crates |

## Findings

### REQUIRED

1. **R20: a purrdf mention outside the recorded allowlist.** `docs/capabilities/census-reference-scan.md` lines 54, 59, 75 and 78 use `purrdf-core` / `purrdf_core::iri::parse` as the reference package in the Usage and Examples sections. The R20 row requires `git grep -il purrdf` to equal exactly the allowlist recorded in `validation.md`. That allowlist is `docs/BRAND.md` only, and the plan's category for census docs covers only an Explanation that names the origin. The row defines any other file as a defect.
   - Preferred fix: replace it with a neutral example package (for example `acme-core` / `acme_core::iri::parse`) in the TOML and the REFERENCE-COPY sample. Then `git grep -il purrdf` returns only `docs/BRAND.md`.
   - Alternative: if the owner wants the real sibling crate as the example, record this file and its justification in the `validation.md` R20 allowlist.

### ADVISORY

1. `validation.md` has no T8 row (T8-review is PASS, but the index stops at T7 plus the T6a test row). It also does not record the R37 hits (AGENTS.md and CLAUDE.md no-deferral rules; AGENTS.md:7 "defers to") as the allowlist that the R37 row asks for. Add both before handoff so the index matches the evidence.
2. The issue text still says `rust-version = "1.97.1"` and lists `.goals`/`.baseline` as deliverables. The branch follows the user decisions (1.98.1; the internal files are ignored). The PR body and issue update should state both amendments explicitly, as planned in T9.
3. R33 (`make check` on the final tree, and CI green) is not evidenced yet. It needs the separately running `make check` result in `validation.md`, then `gh pr checks` after the PR.
4. The analysis R16 literal grep for `reference scan` misses because SECURITY.md says "Reference-package scanning". The content is present, so no change is needed. The R15 "name-only" diff also shows a purrdf-noun sentence adapted (library/codecs, now crates/gates); this matches the issue's "remove purrdf-only content".

VERDICT: BLOCKED

## Recheck

This recheck covers R20 only, at head d6b6572 (`docs(capabilities): use a neutral example package in the reference scan spec`). The commit is signed (`%G?` = G), its subject is conventional, its message has no attribution text, and it changes one file, +4/-4.

- `docs/capabilities/census-reference-scan.md` lines 54, 59, 75 and 78 now use `acme-core` / `acme_core::iri::parse`, consistently across the ledger TOML, the job home and the REFERENCE-COPY sample.
- `git grep -il purrdf` returns only `docs/BRAND.md`, which equals the allowlist recorded in `validation.md`. R20 is satisfied.
- Affected checks after the fix:
  - `cargo test --test docs_contract --locked`: 46 passed, 0 failed.
  - `make issue-refs`: rc 0.
  - `uvx reuse@6.2.0 lint`: compliant, 77/77.
  - The tree is clean.
- The advisories are also resolved in `validation.md`:
  - It now has the T8 row and the R37 hits.
  - It records the full `make check` on 9f93e5d: rc 0, log at `raw/make-check-9f93e5d.log`, which ends with `make check rc=0`.
  - The reuse rationale for the one-file Markdown change is sound.
- Hosted CI (part of R33) remains pending until the PR is opened. It is a T9 gate, not a completeness omission.

The REQUIRED finding is resolved, and no new findings came up.

VERDICT: PASS
