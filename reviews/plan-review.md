# Plan final review (Stage 1)

- **Reviewed plan:** `.stage/raf-workplan-model-documents-check/plan.md`, revision 3.1
- **Reports examined:** `reviews/plan-completeness.md` (F1–F8, A1–A2, N1–N3; PASS on recheck 2), `reviews/plan-constraints.md` (B1–B5, A1–A5, N1–N5; PASS), `reviews/plan-design.md` (B1–B4, A1–A6, N1–N4; PASS), `reviews/plan-enhancement.md` (items 1–17; PASS), `reviews/plan-adversary.md` (F1–F8, N1–N2; ESCALATE-TO-HUMAN on pass 3, resolved by the owner's G21 decision)
- **Dispositions examined:** the G1–G21 table, "Enhancements declined", the rule inventory, "Generated content", the completeness contract and Tasks 1–10
- **User decisions quoted in the plan:** G20 "Citation fix, no reopen (Recommended)"; G21 "Refuse mixed endings (Recommended)"
- **Also read:** `issue.md`, `issue-analysis.md`, `prior-art-assessment.md`, the two capability documents, `okf-profile.md`, and, to check the revision 3.1 text, `crates/raf-okf/src/profile/violation.rs`, `docs/capabilities/okf-profile.md` (frontmatter), `CHANGELOG.md` (sections), `work/changes/` (listing), and the `examples` lists of the two delivered capabilities.

## Fixes are real

Every BLOCKING finding of the five lenses has matching plan text, not just a claim. Spot checks on the items that moved in revision 3 and 3.1:

| Finding | Plan text |
|---|---|
| completeness F1, constraints B4, design B1, enhancement 11 (write path) | G6, Task 5 Check, R14 row ("Running `make docs-sync` twice on this repository writes nothing the second time") |
| completeness N1 (plain-Markdown rules) | G4: `raf_workplan::plain::check(path, text, tracked)`, "marked kept in the disposition table" |
| completeness N2 (exit code) | Task 5: "exits 0 after a successful sync. `--check` ... exits 1" |
| constraints B2, design B3 (link oracle) | G4: `MemoryTree` from `tracked_paths()` |
| constraints B3, design A5 (one pattern set) | G5, Task 3, AGENTS.md in list D |
| constraints N3 (source-index citation) | G20, verbatim, with the PR-body statement |
| constraints N4 (hook-safe commits) | Task 9 Commits 1 and 2 |
| constraints N5 (stage files) | `$STAGE/tasks/...`, "under the git-ignored stage directory" |
| design N1 (fuzz package) | G7: roots `["."]`, exact five-package assertion |
| design N2 (dead `affects-section-missing`) | dropped; Task 2 Check routes it to raf-okf |
| design N3 (two-way coverage test) | G2: `const` members of `KINDS`, both directions |
| adversary F3 pass 2 (one kind per raf-okf rule) | G2: `okf-<rule>`, parse-error kinds, `Rule::ALL` (see B1 below) |
| adversary N1 (split kinds) | inventory rows `depends-missing` / `changes-missing` / `moved-missing` and five `state-*` kinds |
| adversary N2 → G21 | G13, G21, U lines 230–232, inventory `mixed-line-endings`, R14 "CRLF, LF and mixed" (see B2 below) |
| adversary pass-3 advisory (state kinds duplicating raf-okf) | inventory line 86 now routes empty-section and binding minima to `okf-section-empty`, `okf-binding-required`, `okf-binding-not-empty`, consistent with G3 |
| enhancement recheck wording (item 8 reason, G7 "as needed") | "Enhancements declined" cites the open questions; G7 names the roots |

## Declines are justified

`impact` (every constituent query delivered here; only the aggregate's output contract remains with reconcile), the two-way coverage queries (open design questions would be pre-empted), doctest-pairing generalisation (repository policy superseded by the completion gate's example rules) and enhancement 15 (doc-text parsing declined for index facts, with the layering reason in G9). No TRANSFORMATION item was declined. Each decline names a technical reason and the cost.

## Findings

### B1 — BLOCKING — the raf-okf edit in G2 has no owning task and no workflow reading

Plan line (G2): "raf-okf gains a public `Rule::ALL` list, with a test in raf-okf asserting it names every variant through an exhaustive match inside the crate."

Task 1 Scope: "`crates/raf-workplan` (Cargo.toml, `lib.rs`, `finding.rs`, `load.rs`, `model.rs`) and the workspace member." No task names `crates/raf-okf`, its test, or a CHANGELOG line for it. `cap:okf-profile` is `delivered` (`docs/capabilities/okf-profile.md:8`), and the plan's own G20 shows that an edit to a delivered capability's crate or document needs a stated reading. Two facts the implementer also needs: `Rule` is `#[non_exhaustive]` and most variants carry `String` fields (`violation.rs:33-97`), so the list is either variant names or sample values with empty payloads, and G2 should say which; and `raf_okf` already uses the `ALL` idiom for `State`, `CapabilitySection` and `DocType` (`typed.rs:22,68`, `schema.rs:114`), so the shape has a precedent.

REWRITE TO (Task 1 Scope, new bullet): "`crates/raf-okf`: `Rule::ALL`, a list of sample values with empty payloads (the idiom of `State::ALL` and `DocType::ALL`), with an in-crate test matching every variant exhaustively; and its line under `## [Unreleased]` in the CHANGELOG."

REWRITE TO (new G22): "**The owner's decision (verbatim): <...>.** `Rule::ALL` is an additive public constant on raf-okf that changes no documented behaviour of `cap:okf-profile`, so the capability stays `delivered` and no change record is opened. The PR body states this reading." (See open decision 1; the alternative that touches no raf-okf code is the adversary's `okf-unmapped` wildcard kind, which detects a new variant only when a fixture produces it.)

### B2 — BLOCKING — G13 and G21 disagree on which line ending a document "has"

Plan line (G13): "Sync writes each generated block in the document's own line ending, detected from its first line break". Plan line (G21, and U line 230): "A document whose body mixes CRLF and LF is an error".

The profile allows a CRLF frontmatter over an LF body (`okf-profile.md:115-116`: the frontmatter follows the opening fence, the body is kept byte for byte). Such a document is profile-valid and not "mixed" under G21, since its body is uniform. G13 detects CRLF from the fence line and writes a CRLF block into the LF body, so the synced file then fails `check` with `mixed-line-endings`. That contradicts the delivered Staleness sentence "sync makes it pass" and the R14 idempotence check. The CRLF, LF and mixed fixtures in R14 cannot catch it because none has a fence ending different from its body.

REWRITE TO (G13): "Sync writes each generated block in the line ending of the document's **body**, detected from the body's first line break (the frontmatter's ending is irrelevant, as the profile keeps the two independent). A body with no line break takes LF." Add to the R14 row: "a CRLF-frontmatter, LF-body document keeps an all-LF body after sync and passes `check`."

### B3 — BLOCKING — whether a missing `docs/capabilities/index.md` is a finding is unstated, and Task 7's acceptance depends on it

Plan lines: G6 "or the new `docs/capabilities/index.md`"; G12 "the first sync writes the two SPDX HTML-comment lines ... and the `catalogue` block"; Task 9 step 2 "Run `make docs-sync`, which creates `docs/capabilities/index.md`"; Task 7 Check "`cargo test --locked --test docs_contract --test repo_gates` passes on this repository".

At Task 7 the repository has no `index.md` (it is created in Task 9). For Task 7's check to pass, a missing index must not be a finding. But then `sync` has an edit (it creates the index) while `check` is clean, and `docs_sync --check` ("for CI") exits 1 on a repository the Gate passes. That breaks G6's "they cannot disagree" and the inventory has no kind for it. The implementer must choose, and the choice changes Task 7's commit.

REWRITE TO (recommended; see open decision 2): add to G12: "A repository without `docs/capabilities/index.md` is a `check` finding, `index-missing` (fix: run the sync), so `check` and `--check` agree. Fixture: a tree without the index fails with `index-missing`; the same tree after `sync` passes." Add `index-missing` to the inventory row with `generated-stale`. Add to Task 7 Scope: "Run `make docs-sync` once so the repository gains `docs/capabilities/index.md` (both workplan capabilities still `designed`); the file is in this task's commit." Task 9 step 2 then reads "Run `make docs-sync`, which refills every block, including the catalogue".

### A1 — ADVISORY — `state-implementing-no-record` duplicates `no-open-record`

Plan lines: inventory line 86 lists `state-implementing-no-record`; line 87 lists `no-open-record`, "a non-delivered capability with no open change listing it". An `implementing` capability with no open record matches both rules, so one defect yields two findings, against G2's one-vocabulary intent and design principle A. REWRITE TO: delete `state-implementing-no-record`; keep `no-open-record` for all three non-delivered states, and name it as the `implementing` rejected fixture in the R22 row.

### A2 — ADVISORY — Task 5 renders the catalogue before Task 6 builds the `Roadmap` it renders

Plan lines: Task 5 Scope "The G10 renderers" (which include `catalogue`) and Check "The exact-output fixture per kind"; Task 6 Scope "`Roadmap` (G18). `status()` returns it, and the catalogue renders it." The Task 5 implementer must either build `Roadmap` unasked or stub it. REWRITE TO: swap the two tasks (Roadmap and `status()` first, then generated blocks and sync), or open Task 5 Scope with "`Roadmap` (G18), which the catalogue renders" and reduce Task 6 to "`status()` returns the `Roadmap`; the R16 fixtures and the rows-equal fixture".

### A3 — ADVISORY — how `docs_sync` supplies the `SyncOptions` SPDX text is unstated

G12 refuses a sync whose copyright and licence text are unset; Task 5 describes the example as taking only an optional root. REWRITE TO (Task 5): "`examples/docs_sync.rs` takes `--copyright <text>` and `--license <id>` and refuses to run without them; `make docs-sync` passes this repository's values."

### A4 — ADVISORY — G8 states only half of enhancement 13

G8 makes a missing or empty `docs/capabilities` an error. The adopted item also said a missing `work/changes` or `docs/architecture` is an empty set, so a consumer with no open change passes; the plan does not say, and `Tree::list` on a missing directory could go either way. REWRITE TO (G8, append): "A missing `work/changes` or `docs/architecture` directory is an empty set, not an error, with a fixture."

### A5 — ADVISORY — two small ledger slips

- R24 row names Task 2, but "the raf-okf id fixtures surfaced" are Task 1's end-to-end fixtures. REWRITE TO: "| R24, R25 | ... | 1, 3 |".
- List D names `okf-profile.md:31-40, 173-174`; U also rewrites `okf-profile.md:55`. REWRITE TO: "`docs/architecture/okf-profile.md:31-40, 55, 173-174`".

## Integration and coherence

- No contradiction found between G1–G21 other than B2, nor between the rule inventory, the completeness contract, the generated-content spec and the tasks other than A1 and A2. The G3 principle now holds across the inventory (single-document minima are `okf-*` kinds).
- Task order 1–10 is otherwise executable: every rule exists before Task 7 deletes its `docs_contract` copy; `preview` (Task 8) needs only the Task 2 graph; Task 9's two commits are each hook-consistent. Task 10 is the PR task; there is no Task 0.
- `work/changes` keeps seventeen other records after Task 9 deletes two, so the loaded set stays non-empty on this repository.
- The two delivered capabilities bind no `examples`, so G9's new role rule does not threaten G16 at Task 7.
- Each task has its Check line, and the task-structure block supplies review, commit, push and issue update.

## Open user decisions

1. **raf-okf `Rule::ALL` (B1).** Choose one: (a) "Additive constant, no reopen" — raf-okf gains `Rule::ALL` and `cap:okf-profile` stays `delivered`, stated in the PR body like G20 (recommended: it is the only form that fails a test when raf-okf gains a variant); or (b) no raf-okf edit — the wildcard arm maps to an `okf-unmapped` kind excluded by name from the coverage test, which detects a new variant only at runtime.
2. **Missing catalogue index (B3).** Choose one: (a) "Missing index is a finding" — `index-missing`, created in Task 7 by `make docs-sync` (recommended: keeps `check`, `sync` and `--check` in agreement); or (b) "Missing index is clean" — `check` passes without an index and `sync`/`--check` create it only when `SyncOptions::create_index` is set, which adds an option and a second fixture.

VERDICT: NEEDS-REVISION

## Recheck

- **Reviewed plan:** `.stage/raf-workplan-model-documents-check/plan.md`, revision 3.2. I re-read the whole plan and checked each disposition against its text.
- **New user decisions quoted verbatim:** G22 "Additive constant, no reopen (Recommended)"; G23 "Missing index is a finding (Recommended)". Both match open decisions 1(a) and 2(a) above, so no decision remains open.

### Prior findings

| Finding | Status | Evidence (revision 3.2) |
|---|---|---|
| B1 raf-okf edit unowned | **DISCHARGED** | Task 1 Scope: "raf-okf: add `Rule::ALL` with its exhaustive in-crate test (G22)". G22 names the empty-payload form ("the `State::ALL` / `DocType::ALL` idiom"), the reading ("additive API with no contract effect, so `cap:okf-profile` stays `delivered`") and the PR statement. The CHANGELOG has a single `## [Unreleased]` section, so Task 1's "The CHANGELOG entry" covers the raf-okf line. |
| B2 G13 versus G21 | **DISCHARGED** (rule); fixture residue in R1 | G13: "detected from the first line break of the document's **body** (not the frontmatter fence), so a CRLF frontmatter over an LF body gets LF blocks and stays clean". The contradiction is gone. The coordinator's message says a CRLF-frontmatter/LF-body fixture was added, but G13 still ends "Fixtures: CRLF, LF and mixed" and the R14 row is unchanged; see R1. |
| B3 missing index undefined | **DISCHARGED** | G23 records the decision; the inventory row now opens with `index-missing` ("each a `Finding` kind with a fixture beside a valid neighbour"); Task 7: "The first `make docs-sync` run creates `docs/capabilities/index.md` here (G23)". Two stale lines remain downstream; see R2 and R3. |
| A1 duplicate state kind | **DISCHARGED** | Inventory line 88 lists four `state-*` kinds; `state-implementing-no-record` is gone, and `no-open-record` (line 89) covers every non-delivered state. |
| A2 Task 5 before Task 6 | **DISCHARGED** | Task 5 heading "(after Task 6)"; Task 6 Scope: "**Task 6 is implemented before Task 5**, so the catalogue renderer in Task 5 consumes it." Numbering is kept, which is acceptable since both headings state the order. |
| A3 `SyncOptions` text source | **DISCHARGED** | Task 7: "SPDX text taken from `SyncOptions` fields that the `docs_sync` example fills from the repository's own header (copyright and licence) via explicit arguments". |
| A4 G8 half of enhancement 13 | **DISCHARGED** | G8: "A missing `work/changes` is an empty set of changes, not an error". (`docs/architecture` is not mentioned; the same reading follows and no consumer case in the plan depends on it.) |
| A5 ledger slips | **DISCHARGED** | R24/R25 row: "1, 3". List D: "`docs/architecture/okf-profile.md:55`". |

### Residues from the revision 3.2 text (all ADVISORY)

#### R1 — ADVISORY — the body-ending rule has no fixture of its own

Plan line (G13): "Fixtures: CRLF, LF and mixed." None of the three has a frontmatter ending different from its body, so the sentence "a CRLF frontmatter over an LF body gets LF blocks and stays clean" is unfalsified. REWRITE TO (G13, last sentence, and the R14 row): "Fixtures: CRLF, LF, mixed, and CRLF-frontmatter over LF-body, whose body stays all-LF after sync and passes `check`."

#### R2 — ADVISORY — Task 9 step 2 still says the index is created there

Plan line (Task 9 step 2): "Run `make docs-sync`, which creates `docs/capabilities/index.md` (G12) and fills every block." Under G23 the index exists from Task 7. REWRITE TO: "Run `make docs-sync`, which refills every block, including the catalogue in `docs/capabilities/index.md`. Run it again: no change."

#### R3 — ADVISORY — the `INVENTORY` edit for `index.md` is listed under Task 9, but the file now arrives in Task 7

Plan line (list D, Task 9): "the `INVENTORY` table in `tests/docs_contract.rs` (state changes, removed change records, the new `index.md`)". If `INVENTORY` is the exact inventory of `docs/capabilities` (`okf-profile.md:36-37`), the hook's `docs_contract` run fails on Task 7's commit unless the `index.md` row is added in that commit. REWRITE TO: move "the new `index.md`" from the Task 9 D-list entry into Task 7's Scope ("and the `INVENTORY` row for `docs/capabilities/index.md`, in the same commit"), leaving Task 9's entry as "(state changes, removed change records)".

### Coherence

The task order is now executable as written (1, 2, 3, 4, 6, 5, 7, 8, 9, 10), each task has its Check, the inventory and the completeness contract agree with G1–G23, and the two new user decisions are recorded verbatim with their consequences in the tasks. The three residues are wording and fixture-list edits that an implementer would resolve in the right direction from G13 and G23; none changes scope, a contract, or a user decision.

VERDICT: PASS
