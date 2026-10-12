# Plan review (Stage 1, plan-only)

**Reviewed plan:** `/home/paudley/Active/rust-agentic-foundations/.worktrees/1-scaffold-workspace-project-docs-and/.stage/scaffold-workspace-project-docs-and/plan.md`
**Reviewed on:** 2026-10-11
**Inputs read:** `plan.md`, `issue.md`, `issue-analysis.md` (R1–R44, A1–A19, K1–K12), `prior-art-assessment.md`, the approved design plan `/home/paudley/.claude/plans/i-would-two-create-snoopy-sifakis.md`. Nothing under `~/stage` or `~/Active/opencode` was opened.

## Coverage

One focused pass over four concerns, then the four-ground rubric.

| Concern | What was checked | Result |
|---|---|---|
| Completeness | Every issue checklist line and every R1–R44 against the plan's completeness table and task validations | 29/29 issue checklist lines mapped; 42/44 requirements fully mapped; R30 (link resolution) and R36 (process-reference phrasing) only partially mapped (findings 2, 4) |
| Constraints | House rules, `.baseline`, `.goals`, issue "Out of scope", approved-plan conventions, GROUNDING coverage duty, attribution policy | One unresolved conflict (finding 1); one licence-rule inconsistency (finding 3); one missing content constraint (finding 5) |
| Design | A1–A19 decisions, the root-package choice, the docs-contract test, licence layout, OKF profile, task ordering | Sound overall; A13 needs one rule, okf-profile needs two more recorded decisions |
| Utility | Can an implementer execute each task with its listed checks and nothing else? | Yes for T1–T10; four acceptance cells rely on "reviewed" where a mechanical check exists (rubric ground 1) |

Governing text quoted where a finding rests on it:

- Issue, acceptance: "`make check` passes on the empty workspace, and so does CI"; "No issue numbers in repo docs (the `.baseline` rule); identities are slugs". Issue, out of scope: "Any crate code".
- `.baseline` (purrdf text the plan adopts, A5): "You MUST NEVER place github issue numbers, PR# or reference development in the ontology or any documentation in the repo."
- `.goals` (purrdf text the plan adopts): "LOW/NO OPTIONALITY, HARD FAILS".
- Approved plan: "Issue numbers appear only in GitHub. The repo docs refer to change slugs"; "No attribution text anywhere"; "SPDX headers on every file (code `MIT OR Apache-2.0 OR MulanPSL-2.0`, documents CC-BY-4.0)"; "Outside those blocks, the text states the current contract only."
- GROUNDING coverage duty: "every surface a design doc promises is realized, explicitly gated as design-only in the doc itself, or carried by a wired issue."
- Prior-art assessment, risk 7: "The capability docs' Explanation sections should state provenance without implying AGPL-derived code under a permissive grant. The code concern belongs to the later census issues; the doc wording belongs here."

## Findings

### 1. REQUIRED — Committed `.stage/` collides with every repo-wide grep gate and with the issue-number rule; the plan does not scope it

Plan line 6: "It is git-tracked: stagectl's policy is that it is the record, committed with the work in its own commits". Plan line 63: "`.gitignore`: `/target`." Task 9 commits `.stage/scaffold-workspace-project-docs-and/`.

The stage tree contains `issue.md` ("# Issue #1"), `plan.md` ("issue #1", "issues #2–#5", "Task 1", "the plan's"), `issue-analysis.md`, `raw/issue.json` and `brief.json`. Once tracked, these files are inside the scope of:

- the Makefile `issue-refs` target (R36 regex `#[0-9]{1,5}\b|/(issues|pull)/[0-9]+|\bCHANGE-[0-9]+`), which the plan puts in `make check` and in the pre-commit hook (T3). As planned, `make check` fails after T9, or the T9 commit is refused by the hook the plan installed in T3;
- the R20/R43 grep `git grep -niE 'purrdf' -- ':!docs/LICENSING-BACKTRANSLATION.md'` (hundreds of hits from the analysis and prior-art files);
- the R37 deferral grep, the R39 attribution grep, the R43 casing grep.

The plan also silently changes R40 (the analysis requires `/.stage/` ignored; the plan tracks it) without recording the deviation in the completeness table.

Whether a committed process record counts as "documentation in the repo" under `.baseline` is a user call. The plan must either get that call or state the scoping it assumes, and every gate and acceptance grep must carry the scope explicitly.

`REWRITE TO:` (plan line 6, append) "Scope rule: `.stage/` is the process record, not repository documentation. The `.baseline` documentation rules (issue numbers, development references) and the brand/licence/deferral greps apply to the tree **excluding** `.stage/`. The Makefile `issue-refs` target, the pre-commit gate and the `docs_contract` test exclude `.stage/` by path (`':!.stage'`), and `AGENTS.md` records this scope. Confirm with the user before T3 that a tracked `.stage/` is acceptable under the no-issue-numbers rule; if not, `.stage/` goes into `.gitignore` and T9 is removed."
`REWRITE TO:` (completeness table, R40 row) "R40 `.gitignore`: `/target` only. Deviation from the analysis: `.stage/` is tracked, not ignored (stagectl policy); `reuse lint` passes because `REUSE.toml` annotates `.stage/**`."
`REWRITE TO:` (R20/R43 row and every repo-wide grep in the table) add `':!.stage'` to each `git grep` pathspec.

### 2. REQUIRED — The `issue-refs` gate covers only numbers; `.baseline` also bans "reference development"

Plan line 107: "| R36 / R38 no issue numbers, slug ids | T3 (Makefile grep), T6–T8 (test) | `make check` issue-ref grep rc 0; docs_contract test |". Plan line 131 lists the hook gates as "fmt, reuse, issue-refs".

The analysis R36 has two greps: the numeric pattern and the process-reference pattern (`"\b(this|the) issue('s)?\b|this branch|the plan('s)?\b|\btask [0-9]+|\bAC[0-9]+\b"`). The second is the one the essay migration (K5) is most likely to trip: the essay and the approved plan are full of "the plan's", "Task N" and "this issue". The plan never says the Makefile target includes it.

`REWRITE TO:` "| R36 / R38 | T3, T6–T8 | the Makefile `issue-refs` target runs both R36 greps (numeric references and process-reference phrasing) over `git ls-files ':!.stage' ':!*.png'`, rc 0; the hook runs the same target on the index snapshot; the docs_contract test applies the numeric pattern to `docs/` and `work/` |".

### 3. REQUIRED — A13 keeps purrdf's two-licence split for documents, which makes LICENSING.md's own statement false

Plan line 55: "Files copied from purrdf keep purrdf's choice (CONTRIBUTING, CODE_OF_CONDUCT, SECURITY and logos under the code licence)." Plan line 77 (R10): LICENSING.md states "CC-BY-4.0" for documentation, and the approved plan says "documents CC-BY-4.0".

With A13 as written, three documentation files carry the code licence while `LICENSING.md` says documentation is CC-BY-4.0. The prior-art assessment (risk 4) warned exactly this: "Pick one rule ... and apply it consistently. Otherwise the inconsistency gets copied." The plan copies it. Two further facts favour one rule: Blackcat owns all three texts, so relicensing them is free; and the Code of Conduct's `## Attribution` section credits upstream text whose licence is CC-BY-4.0, so the code licence is the less accurate choice for it.

`REWRITE TO:` "A13 licence rule: every `*.md` in the repository is CC-BY-4.0 (including CONTRIBUTING, CODE_OF_CONDUCT, SECURITY, the PR template and `.stage/**`). Logo assets (`docs/*.svg`, `docs/*.png`) are brand assets under the code licence `MIT OR Apache-2.0 OR MulanPSL-2.0`, stated once in `docs/BRAND.md` and in a `REUSE.toml` comment. Code, config, YAML templates and dotfiles use the code licence. R19 check: `git grep -L 'SPDX-License-Identifier: CC-BY-4.0' -- '*.md'` prints nothing."

### 4. REQUIRED — R30 link resolution and R32 cycle detection are not mapped to any task validation

Plan line 103: "| R30 architecture docs | T6 (normal-form), T7 (okf-profile) | files exist with `type: Architecture`; the test validates them |". Plan line 105: "| R32 depends integrity | T6–T8 | the docs_contract test resolves every `depends` id to an existing capability |".

The analysis R30 also requires that capabilities link to the architecture pages (`census-normal-form.md` to `architecture/normal-form.md`; `okf-profile.md` and `workplan-document-model.md` to `architecture/okf-profile.md`) and that every relative Markdown link under `docs/` resolves. R32 also requires no cycles (`tsort`). The docs_contract test scope (plan line 62) lists neither. With 23 cross-linked documents written by hand, dangling links are the likeliest defect and nothing in the plan catches them.

`REWRITE TO:` (plan line 62, test scope) "... `depends` integrity (every target exists and the graph is acyclic), every relative Markdown link under `docs/` and `work/` resolves to a file, each M2 `considered` capability having its change record, every `[!PROPOSED change:<slug>]` marker naming an existing change record, and the issue-reference ban on docs." and (R30 row) "files exist with `type: Architecture`; the test validates frontmatter and links; the R30 `grep -l` checks show `census-normal-form.md` links `architecture/normal-form.md` and `okf-profile.md` plus `workplan-document-model.md` link `architecture/okf-profile.md`".

### 5. REQUIRED — T6 lacks the provenance-wording constraint the prior-art assessment assigns to this issue

Plan line 145: "Content source: the approved plan's raf-census design, and the census survey (katamari `crates/katamari-helper-census`, purrdf `crates/helper-census`, purrdf `helpers-ledger.toml` header, sparrow `docs/helper-census.md`)." Plan line 65 escalates the AGPL relicensing decision but gives no instruction for the documents this issue writes.

katamari and sparrow are `AGPL-3.0-only`. The census capability docs are new CC-BY-4.0 prose. They may describe the design; they must not copy text from the AGPL-labelled files, and their Explanation sections must not state that `raf-census` code derives from those copies, since that would be a licensing claim the user has not made.

`REWRITE TO:` (T6, add a bullet) "Provenance rule: the katamari and sparrow sources are read for facts only; no sentence is copied from an AGPL-labelled file. Explanation sections name purrdf's `helper-census` as the origin of the design and describe the seven copies' drift as the problem; they do not say which copy the code will be ported from. The relicensing decision stays with the user (see 'Not decided here')."

### 6. ADVISORY — Apply the A15 hard-fail-with-hint rule to `rust-validate` as well

Plan line 44 sets `MSRV_RUN ?= rust-validate 1.97.1` and line 57 gives `reuse` a hard failure with an install hint. The Makefile `msrv` target should fail the same way when `$(MSRV_RUN)` is missing, naming `rust-validate` and the CI alternative (`RUSTUP_TOOLCHAIN=1.97.1`), so a contributor without the tool gets a message, not a shell error.

### 7. ADVISORY — Add SPDX headers in T1, not T2

Plan line 126 adds "SPDX headers on the Task 1 files" in T2. The licence expression is already decided, so T1 can ship them, and every commit on the branch is then REUSE-clean (useful when the hook from T3 is installed and when bisecting).

### 8. ADVISORY — `deny.toml` base

Plan line 35 names lillith_memetics; the analysis (R4) found that template has no `[licenses]` section and fails, and the prior-art assessment prefers visual-hashing (SPDX header, `MulanPSL-2.0` in `allow`, `unused-allowed-license = "allow"`). Name the base and the additions in T1 so the R4 check passes first time.

### 9. ADVISORY — Name the CI `reuse` mechanism and the action-pin set

A3 says "CI also runs `reuse lint`" without saying how (`fsfe/reuse-action` at a SHA, or `pipx run reuse lint`). Prior-art risk 13 asks for one SHA-pinned action set (purrdf's `checkout@9c091bb… # v7.0.0` or logit-loom's `3d3c42e… # v7.0.1`). Record both choices in T3.

### 10. ADVISORY — `cliff.toml` `tag_pattern`

purrdf's `rust-v[0-9]*` does not fit RAF. The R18 check only parses the file; decide the pattern (`v[0-9]*`, or per-crate tags) in T4 and record it in CONTRIBUTING's versioning section.

### 11. ADVISORY — Two more decisions belong in `docs/architecture/okf-profile.md`

(a) A11 decides that `designed` capabilities state the designed contract plainly while an open change record tracks delivery, and that `considered` capabilities put the candidate contract in PROPOSED blocks. That state-to-body rule is what the later `check` enforces; T7 lists only the `outcome` optionality. (b) Prior-art 1.12: say whether `id: cap:<slug>` stands in for OKF's `resource` key or coexists with it. Both reduce K11.

### 12. ADVISORY — Cheap fingerprint checks in `make licenses` and the hook

The backtranslation record's own rule ("A change to the Chinese source requires updating this record") has no gate in purrdf (prior-art risk 6). One `sha256sum` compare of `docs/LICENSING.zh-Hans.md` against the hash in `docs/LICENSING-BACKTRANSLATION.md` in the `licenses` target closes it. The Mulan digest check is equally cheap and can join the hook's fast gates (plan line 131 lists fmt, reuse, issue-refs only).

### 13. ADVISORY — Write Explanation sections in T6/T7, verify coverage in T8

T8 says "Explanation sections across the docs absorb the essay", which means editing 16 docs a second time. Writing each Explanation with the essay in hand during T6/T7, and leaving T8 to run the R31 coverage map, delete the file and write the M2 docs, avoids a second pass over every document.

### 14. ADVISORY — Per-task commit subjects and the manual social-preview step

Only T1 and T9 name their commit subjects. R44's regex admits `build|ci|docs|chore|...`; list one subject per task so the K9 slicing is explicit. Also add to the T10 handoff notes that GitHub's social preview must be uploaded by hand in repository Settings (prior-art 1.11: no API), since the PNG alone does not change the repository card.

## Rubric (Stage 1, plan-only)

### Ground 1 — UNFALSIFIABLE CRITERION

Three acceptance cells rest on "reviewed" where the analysis supplies a mechanical check. Severity: ADVISORY each (the plan's table defers to the analysis commands, so the intent is recoverable), but the cells should say the check.

- Plan line 92: "| R20 / R43 no purrdf remnants, brand casing | T4–T5 | `git grep -niE 'purrdf' -- ':!docs/LICENSING-BACKTRANSLATION.md'` lists only intentional provenance mentions (cat-head-core source, adapted-from notes), each reviewed |"
  `REWRITE TO:` "`git grep -nil purrdf -- . ':!.stage' ':!docs/LICENSING-BACKTRANSLATION.md' | sort` equals exactly the allowlist `docs/BRAND.md` (family source) and the census capability docs whose Explanation names the origin; any other file is a defect. `git grep -nwE 'Raf|rAF' -- '*.md' ':!.stage'` prints nothing."
- Plan line 79: "| R12 zh-Hans | T2 | no PurRDF or purrdf; contains RAF; remaining sentences match purrdf apart from the name (diff reviewed) |"
  `REWRITE TO:` "`diff <(sed 's/PurRDF/RAF/g; s/purrdf/raf/g' ~/Active/purrdf/docs/LICENSING.zh-Hans.md) docs/LICENSING.zh-Hans.md | grep -c '^>'` → 0 outside the SPDX and link lines (deletions only, no new Chinese sentences); the R12 purrdf-only-term grep prints nothing."
- Plan line 108: "| R37 no deferrals | T6–T8 | review; outcomes vocabulary contains `implemented | dropped | moved:` only |"
  `REWRITE TO:` "the R37 grep `git grep -niE 'defer|follow-?up|left for future|future work' -- . ':!.baseline' ':!.stage'` returns at most the one documented hit in `workplan-document-model.md`; the `yq` outcome-vocabulary check over `work/changes/*.md` prints nothing."
- Plan line 106: "| R35 current-tense, all sections | T6–T8 | test plus task review (designed or proposed text never phrased as delivered) |"
  `REWRITE TO:` "the R25/R26 loops print nothing; the R35 unresolved-marker grep over `designed` docs prints nothing; the `will|would|shall|should` grep lists candidates and the task review records each surviving hit with its justification (inside a PROPOSED block or an M2 doc)."

### Ground 2 — PRE-EMPTIVE DESCOPE

None found. Every declined enhancement (plan lines 113–118) is outside the issue's checklist: stub crates, a hand-written index, `CLA.md`, `markdownlint`/`codespell`, a YAML dependency. The A1 root package and the docs-contract test add scope rather than remove it, and the issue's "Any crate code" exclusion is respected (no `raf-*` crate; a `publish = false` root package forced by the "`make check` passes on the empty workspace" criterion, with R42 keeping it at zero dependencies).

### Ground 3 — UNMAPPED REQUIREMENT

Two partial gaps, both REQUIRED and already written up with rewrites:

- R30 link resolution and R32 cycle detection (finding 4). Offending line: plan line 103 "files exist with `type: Architecture`; the test validates them".
- R36 process-reference phrasing (finding 2). Offending line: plan line 107 "`make check` issue-ref grep rc 0; docs_contract test".

All other R1–R44 items and all 29 issue checklist lines resolve to a task and an executable check.

### Ground 4 — PLANNED-BY-REFUSAL

None found. Plan line 65 ("Not decided here (for the user): the census crates ... AGPL-3.0-only ... needs your explicit decision before that work starts") is an escalation on work outside this issue, and the plan proceeds without depending on it. R9 "verify only" is correct because the remote is already renamed. A15's "`uvx reuse`" and A14's offline `cargo deny` are executed paths, not refusals.

## Verdict (pass 1)

Five REQUIRED findings. Finding 1 is blocking on its own: as written, the T3 gates and the T9 commit cannot both succeed, and the `.baseline` rule's scope over a tracked `.stage/` is undecided. Findings 2–5 are small, precise edits to the plan text.

VERDICT (pass 1): NEEDS-REVISION

## Recheck (pass 2)

**Reviewed:** the revised `plan.md` at the same path, on 2026-10-11, against the "Plan review dispositions" table (plan lines 181–190). Scope: new text introduced by the fixes and the three user decisions relayed by the coordinator: (a) `.goals`/`.baseline` are internal, git-ignored, never created or committed; (b) `.stage/` is git-ignored and never committed; (c) census code may be ported from any copy, including AGPL-labelled ones, with no notice and no licensing statement. Unchanged text cleared in pass 1 was not reopened.

### Adjudication of pass-1 REQUIRED findings

| # | Pass-1 finding | Disposition | Evidence in the revised plan |
|---|---|---|---|
| 1 | Tracked `.stage/` collides with gates and the issue-number rule | **DISCHARGED** (by user decision b, applied consistently) | Line 6: "`.stage/` ... is **git-ignored and never committed** (user decision). `.gitignore` carries `/.stage/`." Line 63: "`.gitignore`: `/target`, `/.stage/`, `.goals`, `.baseline`." Line 64: the gate "scans the whole tracked tree, with no exemptions needed now that `.stage/` is ignored." The stage-record commit task is gone (tasks now T1–T9, T9 = qualification). Line 88 (R19): "`uvx reuse lint` rc 0 (`.stage/` ignored via `.gitignore`)", which matches the analysis fact that reuse skips ignored files. Line 92 (R8) adds the executable check: "`git check-ignore .goals .baseline .stage/x` matches all three; `git ls-files .goals .baseline .stage` prints nothing." A13 no longer annotates `.stage/**`. The R40 deviation I flagged no longer exists (the plan now matches the analysis's R40). |
| 2 | `issue-refs` covers numbers only | **DISCHARGED** | Line 64 defines pattern set (1) issue/PR tokens and (2) "development-process wording from the analysis's R36 second grep: `this issue`, `the plan's`, `Task [0-9]+`, `AC[0-9]+`, `per the plan`, `this branch`", implemented as `git grep -nE` that "fails with every hit listed". Line 139: the hook's fast gates include `issue-refs`; CI "adds ... an `issue-refs` step". Line 114 (R36) adds a falsifiable negative check: a temp file containing `Per #12` or `this issue` makes `make issue-refs` fail. Line 62: the docs test enforces "the issue-reference and process-wording bans" with negative fixtures. Line 26 ties this to the `.baseline` rule ("no issue numbers or development-process references in repo docs"), so the rule binds even though `.baseline` is not committed. |
| 3 | Inconsistent document-licence split | **DISCHARGED** | Line 55 (A13): "one rule stated once in LICENSING.md and REUSE.toml: **Every `*.md` file is CC-BY-4.0.** This includes CONTRIBUTING, CODE_OF_CONDUCT and SECURITY ... SVG/PNG brand assets, code, config and templates use the code licence". The REUSE-IgnoreStart/End wrapping for quoted SPDX tags (analysis R10 copy hazard) is now stated. LICENSING.md's "documentation under CC-BY-4.0" statement is therefore true of every Markdown file. |
| 4 | Link resolution and cycle detection unmapped | **DISCHARGED** | Line 62 (test scope): "every `depends` id resolves, and the `depends` graph is **acyclic**; every relative Markdown link in `docs/` and `work/` resolves to an existing file (and anchor, for `#section` links to H2s); every `> [!PROPOSED change:<slug>]` names an existing `work/changes/<slug>.md` ... Each rule has a negative fixture". Line 110 (R30): the test "resolves every relative link and anchor in `docs/` and `work/`". Line 112 (R32): "rejects cycles (negative fixtures: a dangling id and a 2-cycle)". Residual (ADVISORY, see P2-3): the two presence checks from analysis R30 (`grep -l` showing the census and workplan docs link to their architecture pages) were not adopted. |
| 5 | AGPL provenance-wording constraint for T6 | **DISCHARGED (superseded by user decision c)** | Line 27 and line 65: "census code may be ported from any existing copy (including the AGPL-labelled ones) with no notice and no relicensing statement (user decision) ... Census capability docs may describe the design's origin freely." Line 72 replaces the pass-1 escalation with "Decided ... Nothing is left open." The constraint I asked for no longer applies. |

**STILL-OPEN items: none.**

### Adjudication of the pass-1 rubric ground 1 rewrites

All four cells now carry mechanical checks: R12 (line 86, the `diff ... | grep '^>'` form), R20/R43 (line 99, allowlist bounded to `docs/BRAND.md` plus census Explanation sections, plus the casing grep), R35 (line 113, TBD/TODO grep over `docs work` prints nothing; the modal-verb grep is enumerated and each hit justified), R37 (line 115, the deferral grep with each hit required to be a rule prohibiting deferral, which correctly accommodates the new `AGENTS.md` statement of the no-deferral rule; outcome vocabulary enforced in the test). Accepted.

### Advisory items applied

Dispositions for 6–14 match the new text: A15/adv. 6 (line 57, `command -v` check and a named alternative for both tools, with an executable negative check in the R5 row), adv. 7 (line 128, SPDX headers in T1), adv. 8 (line 66), adv. 9 (line 67), adv. 10 (line 68), adv. 11 (line 69), adv. 12 (line 70, `make licenses` in `check` and the hook), adv. 13 (lines 157, 164, 169), adv. 14 (commit subjects on T1–T8; line 178 social-preview note). Accepted.

### New text: observations (all ADVISORY, none blocking)

- **P2-1.** Line 155 (T6) still enumerates the old six negative fixtures ("an unknown key, a missing section, sections out of order, a bad state, a dangling `depends`, an issue reference"), while line 62 now says "Each rule has a negative fixture" and adds a 2-cycle, a dangling link, an orphan PROPOSED marker, a bad outcome and a process-wording hit. Line 62 governs; make line 155 say "one negative fixture per rule listed in the docs-contract decision" so the two lists cannot drift.
- **P2-2.** Line 64 describes pattern (1) as "`#[0-9]+` outside hex colour tokens". Use the analysis's exact regex `(^|[^[:alnum:]&/])#[0-9]{1,5}\b`, which already excludes `#111214`-style tokens and Markdown headings, and pass `-I` to `git grep` so the tracked PNG cannot produce a "Binary file matches" hit. (The plan's narrowing from the analysis's `(this|the) issue` to `this issue` is a reasonable decision: it keeps the PR template's "Link the issue if there is one".)
- **P2-3.** R30 row: add the two presence greps from the analysis (`grep -l 'architecture/normal-form.md' docs/capabilities/census-*.md` includes `census-normal-form.md`; `grep -l 'architecture/okf-profile.md' docs/capabilities/*.md` includes `okf-profile.md` and `workplan-document-model.md`). Resolution is now checked; presence of the link is not.
- **P2-4.** Line 57: `REUSE ?= uvx reuse` is two words, so the existence check must test `$(firstword $(REUSE))`. Consider pinning the version (`uvx reuse@6.2.0`) in both the Makefile and the CI step so local and CI results match (prior-art risk 2).
- **P2-5.** Line 66: `Unicode-3.0` in the `deny.toml` allow list has no consumer in this issue (zero dependencies). Harmless under `unused-allowed-license = "allow"`; add a one-line comment naming why it is pre-approved (syn's `unicode-ident`) or drop it until a dependency needs it.
- **P2-6.** The issue's checklist line "`AGENTS.md`, `CLAUDE.md`, `.goals`, `.baseline`, `.deficiencies` from the house templates" will remain partly unchecked by user decision (a). Line 92 records the amendment in the plan; the T9 PR body and the issue update should say so explicitly, so the unchecked box reads as a decision rather than a miss.

### Verdict (pass 2)

All five pass-1 REQUIRED findings are discharged with evidence in the revised text (three by plan edits, two by user decisions applied consistently through the plan). The rubric rewrites are adopted. The six pass-2 observations are advisory refinements to new text and do not block implementation.

VERDICT: PASS
