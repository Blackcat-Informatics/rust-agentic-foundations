# Plan completeness review

- **Reviewed plan:** `.stage/raf-workplan-model-documents-check/plan.md`
- **Inputs read in full:** `issue.md` (title, body, 0 comments), `issue-analysis.md` (R1–R32, 12 NEEDS-JUDGMENT), `prior-art-assessment.md` (7 NEEDS-JUDGMENT), `docs/capabilities/workplan-document-model.md`, `docs/capabilities/workplan-generated-docs.md`, `docs/architecture/okf-profile.md`, `AGENTS.md`, `work/changes/workplan-document-model.md`, `work/changes/workplan-generated-docs.md`. Read the relevant parts of `tests/docs_contract.rs` (its index of tests and binding roles), `crates/raf-source/src/tree/mod.rs` (`Tree`), and `raf-okf` (`replace_spans`, `ProposedOutsideSection`).
- **Ledger audit:** I re-read the issue body. Every requirement maps to an R-id, and the ledger has no misses. Each of the 12 ledger NEEDS-JUDGMENT items and the 7 prior-art items has a disposition (J1–J15).

## Requirement to task to check

| Source | Requirement | Task | Check | Status |
|---|---|---|---|---|
| R1–R3 | Plan, implement and reconcile; one current document; no amendments applied in the reader's head | 2, 3, 8 | State and PROPOSED rules, and the delivered docs pass `check` | OK |
| R4 | Both capabilities delivered | 8 | `state: delivered`, bindings resolve, matrices synced, records completed | PARTIAL (F3: reconcile covers Contract only) |
| R5–R7, R17 | Capability schema, closed keys, seven sections | 1 | One end-to-end raf-okf violation kind per fixture through `check` | OK |
| R8–R10 | Change schema, outcomes, no `deferred` | 1, 2 | `deferred` fails; `moved` naming a missing change fails beside a valid one | OK |
| R11, R21 | PROPOSED blocks | 3 | Four marker fixtures from the Verification section | PARTIAL (F4: the profile's section-in-`affects` and inside-a-section rules are missing) |
| R12 | Graph nodes, edges, queries in both directions | 2 | `Workplan::graph()` fixture | OK |
| R13, R19 | Bindings resolve through raf-source | 4 | Resolve and fail fixtures over a `MemoryTree` | OK (macro-generated case: F4) |
| R14, R15 | Generated blocks; sync rewrites them deterministically | 5 | Staleness, order, markers, outside text, index | PARTIAL (F1: there is no write path; F5: no golden fixture of the matrix or relationships content) |
| R16 | `status` roadmap | 6 | `Workplan::status()` fixture, and a run on this repository | OK (Task 6 has no Check line: F2) |
| R18 | Links resolve | 3, 7 | Ported link fixtures | PARTIAL (F4: J6's wiring for files outside `docs/**` and `work/**` has no check) |
| R20 | Generated blocks in sync | 5 | Staleness fixture per kind | OK |
| R22 | Delivered: no PROPOSED block, no empty section | 3 | Delivered and considered fixtures | PARTIAL (F6: no rejection fixture for `designed` or `implementing`) |
| R23 | `Gate::workspace(..).assert_clean()` | 7 | Clean and broken tempdir, plus `docs_contract` on this repository | OK |
| R24 | IDs are slugs | 2 | Id and file mismatch, non-kebab slug, `affects` slug | OK |
| R25 | Issue-number references rejected | 3 | `#123`, `issue 12`, `PR 5`, URLs; hex colours spared | OK |
| R26 | A fixture for every check failure | 1–5 | Exhaustive-match coverage test over `Rule` | PARTIAL (F4: model rules leave `docs_contract` without a list of where each goes) |
| R27 | Idempotent and byte-deterministic sync | 5 | Fixture: two syncs give equal bytes | PARTIAL (F1: no run on the shipped `make docs-sync`) |
| R28 | This repository passes `check` | 7, 8 | `Gate` in `docs_contract` and in the hook snapshot | OK |
| R29 | Or record the gaps as obligations | — | Not used (J11, R28 chosen) | OK |
| R30 | Capability docs reconciled | 8 | Reconcile Contract sentences | PARTIAL (F3) |
| R31–R32 | Dependencies | — | Closed and merged (J9) | OK |
| DM Contract: four states, change-record column | State rules | 3 | Task 3 text only | PARTIAL (F6) |
| DM Contract: designed and considered carry empty bindings | Binding rule | — | none | UNMAPPED (F4) |
| DM Contract and Verification: `changes` entry naming a missing capability fails | Graph fixture | 2 | Task 2 text only; no contract row | PARTIAL (F7) |
| DM Contract: acyclic `depends` (self, 2-node, 3-node) | Graph rule | 2 | Task 2 text only; no contract row | PARTIAL (F7) |
| DM Contract: `affects` sections exist | Reference rule | 2 | Task 2 text only | PARTIAL (F7) |
| DM Constraints: flat directories | Directory rule | 1 | Task 1 text; the fixture is not named | PARTIAL (F7) |
| DM Verification: all six bullets | — | 1–4, 7 | As above | PARTIAL (F6, F7) |
| GD Contract: block placement per kind (matrix and relationships in a capability, catalogue in `index.md`) | Placement rule | — | none | UNMAPPED (F8) |
| GD Contract: matrix rows (item, kind, section, path order); relationships as links; catalogue row links and description | Content | 5 | No content assertion | UNFALSIFIABLE (F5) |
| GD Contract: a file with no block is not touched; `\n` endings | Sync | 5 | none | ADVISORY (A1) |
| GD Constraints: blocks sit inside a section and do not count toward non-empty | Rule | — | none | UNMAPPED (F8) |
| GD Constraints: preview is printed, never written | Preview | — | none; `docs preview` belongs to `cap:workplan-cli` | UNMAPPED (F3) |
| GD Verification: all six bullets | — | 5 | R14 row | OK |
| Plan structure | Per-task check, review, commit, push, issue update; final PR task; no Task 0 | all | — | STRUCTURE (F2) |

## Findings

### F1 — BLOCKING — STUB (no shipped write path for sync)

Plan lines: J13 "The library gives `Workplan::sync() -> SyncReport`, which writes through the tree." Task 5: "`examples/docs_sync.rs` writes through `DiskTree` for this repository."

`raf_source::Tree` is "A read-only tree of UTF-8 text files below one root" (`crates/raf-source/src/tree/mod.rs:139`). It has `read`, `read_unseen`, `identity` and `list`, and no write method. `DiskTree` cannot write, and the plan never creates a write API. As written, the shipped `make docs-sync` entry point has nothing to call, so the sync half of R15 and R27 and Task 8's "Run `make docs-sync`" cannot be delivered. R27 is also checked only on fixtures, never through the shipped target.

REWRITE TO (J13, and the Sync bullet of Task 5):
> **Sync:** `Workplan::sync(&self) -> SyncReport` is pure. It returns, for each file whose generated blocks change, the `RelPath` and the full new text (raf-okf `replace_spans`), and it returns nothing for an unchanged file. `SyncReport::write(root: &Path) -> io::Result<()>` writes each file in the report with `std::fs`, inside `root`, by a temporary file and a rename, and writes no other file. `examples/docs_sync.rs` loads this repository through `DiskTree`, calls `sync`, then `write`, and prints the changed paths. `make docs-sync` runs it.
> **Check:** the fixtures run over `MemoryTree`, and a second `sync` on the result returns an empty report with byte-equal text. On this repository, in Task 8, `make docs-sync && make docs-sync && git diff --exit-code` passes after the first sync is committed, and the second run prints no changed path.

### F2 — BLOCKING — STRUCTURE (per-task loop is incomplete)

Tasks 2, 3, 4, 5 and 6 have no **Check** line. No task has an independent task review, a push or an issue update. Each task gives only "- **Commit:** ...".

REWRITE TO (append to every task, Tasks 1–8, and fill in the task's own check):
> - **Check:** `cargo test -p raf-workplan --locked` (Tasks 1–6), and `cargo clippy -p raf-workplan --all-targets --locked -- -D warnings`, with this task's named fixtures listed (for example Task 2: `rejects_id_filename_mismatch`, `rejects_non_kebab_slug`, `rejects_changes_naming_missing_capability`, `rejects_affects_section_unknown`, `rejects_moved_outcome_naming_missing_change`, `rejects_depends_self_cycle`, `rejects_depends_two_cycle`, `rejects_depends_three_cycle`, `accepts_acyclic_graph`).
> - **Review:** an independent task review of the diff against this task's contract rows before the commit.
> - **Commit**, then **push** to `origin/paudley/7-raf-workplan-model-documents-check`.
> - **Issue update:** post a comment on the issue naming the task, the commit and the check result.

Task 9 (PR, plan publication, handoff) and the absence of a Task 0 are correct.

### F3 — BLOCKING — SILENT CUT (reconcile is limited to Contract; stale statements elsewhere)

Plan line, Task 8: "reconcile every Contract sentence against the code, removing or fixing any unbuilt claim" and "update the AGENTS.md inventory".

R30 ("Capability docs reconciled") and the document-model Constraint ("A statement about behaviour that is not built belongs in a `designed` or `considered` capability or inside a PROPOSED block, never in a delivered contract") cover every section. Known statements will be false once Task 8 lands:
- `workplan-generated-docs.md` Usage shows `cargo workplan docs sync`, `check` and `docs preview <slug>`, and no `cargo workplan` binary exists. Constraints: "Preview output is printed and never written to a file", but no preview is built. Preview belongs to `cap:workplan-cli` (`workplan-cli.md:51`).
- `workplan-document-model.md` Usage: "The `cargo workplan` binary exposes the same checks".
- `docs/architecture/okf-profile.md:31-40`: the cross-document checks are in the document-contract test, "what `raf-workplan` replaces once it is implemented". Line 174: generated blocks, "None exist yet."
- `docs/capabilities/source-index.md:1497`: "Binding fixtures in the document-contract test", and line 1489. J1 moves those fixtures into `crates/raf-workplan/tests`.
- `AGENTS.md:41-45`, `:63`, `:74`, `:209-223`, and `CONTRIBUTING.md:33`: the document-contract test enforces the format until `raf-workplan` exists; `raf-workplan` is `designed`; `crates/` holds only raf-okf and raf-source.
- `crates/raf-okf/tests/usage_docs.rs:6` names the document-contract test as the place where the cross-document checks run.

REWRITE TO (Task 8, second and third bullets):
> - Set both to `delivered`, with code, test and example bindings to the built items. Reconcile **every section** (Purpose to Verification) of both documents against the code, and remove or fix every unbuilt claim. In `workplan-generated-docs.md`, the Usage section shows `make docs-sync`, `examples/docs_sync.rs` and the `Workplan::sync` and `Workplan::check` API in place of `cargo workplan`. The `docs preview` Usage line and the "Preview output is printed and never written to a file" Constraint move into `docs/capabilities/workplan-cli.md` under its open change `change:workplan-cli`, recorded as `moved:change:workplan-cli` on the matching item. In `workplan-document-model.md`, the Usage sentence about the `cargo workplan` binary becomes a link to the designed `cap:workplan-cli` and claims no binary.
> - Reconcile the documents that describe the old gate: `docs/architecture/okf-profile.md` (the "Relation to OKF" paragraph says `raf-workplan` runs the cross-document checks and `docs_contract` keeps only the repository policy; "None exist yet" is removed), `docs/capabilities/source-index.md` (binding fixtures cited in the `raf-workplan` tests), `AGENTS.md` sections 1, 3 and 6 and the crate and layout tables (`raf-workplan` is a member, with the two delivered capabilities named), `CONTRIBUTING.md`, the `INVENTORY` table in `tests/docs_contract.rs`, and the module doc of `crates/raf-okf/tests/usage_docs.rs`.
> - **Check (added):** `rg -n 'cargo workplan|None exist yet|once it is implemented|once implemented' docs AGENTS.md CONTRIBUTING.md` returns no line in a delivered document, the architecture page or AGENTS.md, except links to `workplan-cli.md`.

### F4 — BLOCKING — SILENT CUT (J1 deletes model rules from `docs_contract` without a list of where each one goes)

Plan lines: J1 "Its existing fixtures for model rules move into `crates/raf-workplan/tests`". Task 7: "model rules and helpers are deleted, along with their fixtures (moved into the crate in Tasks 1–5)".

No task or contract row names several rules that `docs_contract` enforces today, so deleting them in Task 7 would remove enforcement silently:
- `rejects_bindings_before_delivery`: designed and considered lists must be empty (DM Contract, Binding resolution; profile `bindings` value rule).
- `rejects_designed_with_proposed_block`: the profile's state table.
- `rejects_proposed_section_missing_from_affects`: the profile, "appears as `cap:<slug>#<section>` in the `affects` of that record".
- `rejects_open_capability_without_change_record`: DM four-states table.
- `a_binding_to_a_macro_generated_function_is_not_found` and `every_unresolved_binding_of_every_list_is_reported`: DM Contract, Binding resolution.
- `rejects_non_markdown_file_in_a_document_directory`, `rejects_nested_documents`, `refuses_non_regular_files_instead_of_blocking` and `refuses_a_file_swapped_in_after_inspection`: profile directory rules and J14.
- The index variants: `rejects_an_index_with_prose_outside_the_blocks`, `..._marker_is_only_in_a_code_span`, `..._mid_line_marker`, `..._indented_marker`, `..._header_with_unknown_tags...` and `accepts_a_generated_index_with_an_spdx_header`.
- The link forms: `rejects_broken_reference_definition`, `rejects_broken_href_and_src_attributes`, `rejects_a_link_split_across_lines...`, `rejects_an_empty_link_target` and `does_not_read_links_or_placeholders_inside_html_comments`.
- J6's promise that `docs_contract` checks links in the other Markdown files (README, AGENTS.md) with the public checker has no check. `rejects_broken_link_outside_the_document_directories` must stay green.
- J1's keep-list does not say where `usage_and_examples_blocks_are_the_crate_doctests` and its doctest-comparison fixtures go. They are neither model rules nor in the keep-list.

REWRITE TO (add to Task 7 before "model rules and helpers are deleted"):
> - **Port map:** `crates/raf-workplan/tests/PORTED.md` is not committed. Instead, the PR description carries a table with one row per `#[test]` in the current `tests/docs_contract.rs`, each marked **moved** (naming the `raf-workplan` test that replaces it), **kept** (repository policy: inventory, the wording and placeholder scans over tracked files, the tracked-symlink rule, links in Markdown outside `docs/**` and `work/**` through `raf_workplan::links`, and the crate-doctest pairing tests) or **covered by raf-okf** (naming the raf-okf test). A test with none of the three blocks deletion. Each moved rule keeps its negative fixture beside a valid case, including bindings in a `designed` or `considered` capability, a PROPOSED block in a `designed` capability, a PROPOSED block whose section is missing from `affects`, a non-delivered capability with no open record, a macro-generated binding, every unresolved binding reported, a non-Markdown or nested file in a document directory, non-regular and swapped files, the six index variants, and the reference, `href`/`src`, split and empty link forms.
> - **Check:** `cargo test --locked --test docs_contract` still runs `rejects_broken_link_outside_the_document_directories` and `rejects_a_link_split_across_lines_in_any_tracked_markdown_file` and passes. `cargo test -p raf-workplan --locked` runs every **moved** test name in the table.

### F5 — BLOCKING — UNFALSIFIABLE (generated content is never asserted)

Plan line, R14 row: "Per kind: an edited row, a removed row and an added binding each make `check` fail naming the kind, and sync repairs it." The staleness fixtures prove that sync and check agree with each other. They do not prove the Contract's content: a matrix row giving "the bound item, its kind and the section ... it substantiates", sorted by path; relationships listing depends-on, depended-on-by and open changes "each as a link"; a catalogue row that "links to the document and carries its description". A generator that wrote the wrong section, or no links, would still pass.

REWRITE TO (add to the R14, R15, R20 and R27 row and to Task 5):
> Golden fixtures, one per kind, compare the synced block byte for byte with an expected string. `verification-matrix`: a capability with two tests and one example (an example target and a doctest) gives three rows in path order, each with the item path, its kind (`test`, `example`, `doctest`) and its section. A binding named in the Contract text maps to `contract`, and the others map to `verification`. `relationships`: a capability with one dependency, one dependant and one open change gives the table in the form of the generated-docs Examples section, each cell a relative link. `catalogue`: each row links `./<slug>.md` and carries the description.

### F6 — BLOCKING — UNMAPPED (a state fixture rejected per state)

DM Verification: "A state fixture per state: each accepted with matching content, and each rejected with a mismatch". The R22 row gives acceptance per state, but rejections only for `delivered` and `considered`.

REWRITE TO (append to the R22 row):
> Rejected per state: `considered` with no PROPOSED block, or with no open record; `designed` with a PROPOSED block, with a non-empty binding list, or with no open record; `implementing` with no open record listing it; `delivered` with a PROPOSED block, with an empty section (including one holding only comments or only a generated block), with no code or no test binding, or listed by an open record.

### F7 — BLOCKING — UNMAPPED (cross-document rules from Task 2 have no contract row)

The completeness contract has rows only for issue R-ids. The DM Checks list and Verification items for "references that exist" and "an acyclic `depends` graph" ("a two-node and a three-node `depends` cycle fail, and a `changes` entry for a missing capability fails") appear only as Task 2 prose, with no acceptance check. The same holds for "Directories are flat" (Task 1 prose).

REWRITE TO (add contract rows):
> | DM Checks: references exist | `depends` naming a missing capability, `changes` naming a missing capability, `affects` naming a section outside the seven, and `moved:change:<slug>` naming a missing change each fail with the file named, beside a valid document | 2 |
> | DM Checks: acyclic `depends` | An acyclic graph passes; a self cycle, a two-node cycle and a three-node cycle each fail, naming every member | 2 |
> | DM Constraints: flat directories | `docs/capabilities/sub/x.md`, a Change document in `docs/capabilities` and a Capability document in `work/changes` each fail; `docs/capabilities/index.md` passes | 1 |

### F8 — BLOCKING — UNMAPPED (generated-block placement)

The GD Contract says `verification-matrix` and `relationships` go "in a capability document" and `catalogue` "in `docs/capabilities/index.md`". The GD Constraints say: "Generated blocks sit inside a section. They do not replace a required section and do not count toward its being non-empty." raf-okf has `ProposedOutsideSection` but no rule for a generated block, and the plan names none.

REWRITE TO (add to Task 5 and as a contract row):
> **Placement:** a `catalogue` block outside `docs/capabilities/index.md`, a `verification-matrix` or `relationships` block in `index.md`, a Change or Architecture document, and a generated block in a capability document outside the seven sections (before the first `##`) each fail, naming the file and the kind. Each sits beside the same block in its valid place, which passes. A required section holding only a generated block is empty (Task 3 state fixture).

### A1 — ADVISORY — sync contract details without a fixture

GD Contract: "A file with no generated block is not touched" and "`\n` line endings". Add to Task 5: a fixture asserting that `SyncReport` lists no file that has no block, and a CRLF capability document whose synced block is written with `\n`, or one that is refused, as the reconciled Contract states.

### A2 — ADVISORY — stated lesson with no task

Plan line: "Rule: run the gate on this repository and on katamari/purrdf documents where they exist." No task runs it on katamari or purrdf. Either add to Task 8: "Run `Gate::workspace` on the katamari and purrdf checkouts where they hold `docs/capabilities`, and record the result in the PR", or drop "and on katamari/purrdf documents where they exist".

VERDICT: NEEDS-REVISION

## Recheck

- **Reviewed plan:** `.stage/raf-workplan-model-documents-check/plan.md`, revision 2. I re-read the whole plan and checked the new text against `tests/docs_contract.rs:273-321` (`read_plain`) and `README.md:25-61`.

### Prior findings

| Finding | Status | Evidence (revision 2 line) |
|---|---|---|
| F1 sync write path | **DISCHARGED** | G6: "Sync returns texts; callers write. `Workplan::sync() -> SyncReport { edits: Vec<(RelPath, String)> }` ... The `Tree` seam stays read-only ... Writing is done by `Gate`/`Workplan::write(edits, root)` ... temporary file and ... renamed into place". Task 5 Check: "`cargo run -p raf-workplan --example docs_sync -- <tempdir fixture>` writes the expected files once and nothing on a second run". R14 row: "Running `make docs-sync` twice on this repository writes nothing the second time". Task 9 step 2 runs it on the real repository. |
| F2 structure | **DISCHARGED** | "Task structure" steps 2–6: Check, an independent reviewer writing `tasks/TN-review.md`, commit then push, and "Post the task summary to the issue". Every task (1–9) has a **Check** line. There is no Task 0. Task 10 creates the PR, publishes the plan and hands off. |
| F3 reconcile scope | **DISCHARGED** | G15: "Every statement in every section of both documents is either built or listed in U". G14 builds `Workplan::preview`, so the preview constraint is delivered, not cut. Task 9 list U rewords the `cargo workplan` Usage lines, the Constraint, "Mechanical versus explanatory" and the obligations sentence. List D names `source-index.md:1488-1500`, `okf-profile.md:31-40, 173-174`, `AGENTS.md`, `README.md:31`, `CONTRIBUTING.md`, `usage_docs.rs:4-6` and `INVENTORY`. |
| F4 port map | **DISCHARGED**, with one new gap (N1) | Task 7: "every existing test function is listed in `tasks/T7-dispositions.md` as moved (new path::name), kept, or replaced by a raf-workplan rule (kind). No test is dropped without a row, and the reviewer checks that the table covers every `#[test]`", plus the fixture-count check. The rule inventory names `state-designed` (empty bindings), `proposed-in-designed`, `proposed-section-not-affected`, `no-open-record`, `binding-macro-generated`, `stray-markdown`, `nested-directory` and `index-hand-written`. The R18 row adds links outside `docs/` and `work/` through the public checker, "with a fixture for each". G4 keeps the doctest-pairs tests. |
| F5 generated content | **DISCHARGED** | G10 and "Generated content": exact columns, rows, section rule, ordering and links per kind, with an "exact-output fixture asserting the bytes" for each. The catalogue row links the document and carries its description. |
| F6 per-state rejection | **DISCHARGED** | Rule inventory `state-*`: "Each state has an accepted and a rejected fixture", with the `designed` and `implementing` mismatches named. R22 row: "The `state-*` fixtures for all four states." |
| F7 cross-document rows | **DISCHARGED** | Rule inventory rows `missing-reference`, `affects-section-missing`, `depends-cycle` ("self, 2-node and 3-node"), `wrong-directory` and `nested-directory`. The Task 2 Check names the first three, and the Task 1 Check names the directory fixtures. |
| F8 placement | **DISCHARGED** | G11 ("valid only in a Capability document, inside a section ... `catalogue` ... only in `docs/capabilities/index.md`. A generated block does not make a section non-empty. Each rule gets a fixture"). The inventory has `generated-misplaced`, and the R14 row has "Placement (G11)". |
| A1 | **DISCHARGED** | R14 row: "A file without blocks is untouched. Every edit ends in `\n` with no `\r`. CRLF (G13)." |
| A2 | **DISCHARGED** | G19 drops the katamari/purrdf line, with the reason. |

### New findings

#### N1 — BLOCKING — SILENT CUT (G4's keep-list drops the plain-Markdown rules)

Plan line, G4: "`docs_contract` keeps only this repository's own policy: the `INVENTORY` table, the doctest-pairs tests ... and the tracked-symlink rule."

Today `read_plain` (`tests/docs_contract.rs:273-321`) applies four checks to every tracked Markdown file outside the document directories (README, AGENTS.md, CONTRIBUTING.md, `docs/BRAND.md`, …): split links, placeholder tokens, PROPOSED blocks outside a capability, and generated-marker errors. The tests are `reads_placeholders_in_plain_markdown_outside_code`, `reports_marker_errors_of_plain_markdown_once` and `reads_a_proposed_block_in_plain_markdown_as_outside_a_capability`. G1 limits `raf-workplan` to `docs/capabilities/`, `docs/architecture/` and `work/changes/`. Revision 2 takes over only the link half for the other files (R18 row). The placeholder, PROPOSED-outside-a-capability and marker checks on those files appear in neither the keep-list nor the rule inventory. A Task 7 disposition row could mark them "replaced by a raf-workplan rule" even though no `raf-workplan` rule reads those files. The plan's own lesson says gates must not weaken while code moves.

REWRITE TO (G4, the keep-list sentence):
> `docs_contract` keeps only this repository's own policy: the `INVENTORY` table, the doctest-pairs tests (...), the tracked-symlink rule, and the plain-Markdown checks over the tracked Markdown outside `docs/capabilities`, `docs/architecture` and `work/changes`: links and split links through `raf_workplan`'s public link checker, placeholder tokens, a PROPOSED block outside a capability, and generated-marker errors. They keep their existing tests (`reads_placeholders_in_plain_markdown_outside_code`, `reports_marker_errors_of_plain_markdown_once`, `reads_a_proposed_block_in_plain_markdown_as_outside_a_capability`, `rejects_a_link_split_across_lines_in_any_tracked_markdown_file`, `rejects_broken_link_outside_the_document_directories`), each marked **kept** in `tasks/T7-dispositions.md`.

#### N2 — ADVISORY — `make docs-sync` fails on a successful first write

Plan line, Task 5: "`examples/docs_sync.rs` ... exits 0 when nothing changed and 1 when it wrote." Task 9 step 2: "Run `make docs-sync`, which creates `docs/capabilities/index.md` ... Run it again: no change." With exit 1 on write, the first `make docs-sync` reports a make failure after a correct write. Either state that this is intended (a check-mode exit) and that Task 9 expects exit 1 then exit 0, or have `make docs-sync` exit 0 on a successful write.

#### N3 — ADVISORY — remaining "planned, not available" and `docs sync` wording

- `README.md:34-37` ("The API below is planned and is not available yet") covers `raf_workplan::Gate::workspace(..).assert_clean()`, which this PR builds. List D names only `README.md:31`. Add `README.md:34-37` to D, worded so it stays true: the crate is built in the repository but not published.
- The generated-docs Contract and the document-model Checks still say `docs sync` ("`docs sync` rewrites every generated block", "what `docs sync` would write"). Add a U entry that defines `docs sync` in those sentences as the sync operation (`Workplan::sync` and `write`, run by `make docs-sync`), so no reader takes it for a built command.

VERDICT: NEEDS-REVISION

## Recheck 2

- **Reviewed plan:** `.stage/raf-workplan-model-documents-check/plan.md`, revision 3. I re-read G4, G13, the R14 row, Tasks 5, 7, 8 and 9 (lists U and D), and the "Enhancements declined" line on exit codes.

### Prior findings

| Finding | Status | Evidence (revision 3 line) |
|---|---|---|
| N1 plain-Markdown rules | **DISCHARGED** | G4: "It also keeps the checks on tracked Markdown **outside** the document folders (README, AGENTS.md and so on): links, placeholder tokens, a PROPOSED block outside a capability, generated-marker errors and split links. These run through one public raf-workplan function, `raf_workplan::plain::check(path, text, tracked)`, which shares the rule code with `check` ... The existing tests for them are marked "kept" in the disposition table." All four `read_plain` checks are named, and they keep their existing tests through the Task 7 disposition table. |
| N2 `docs-sync` exit code | **DISCHARGED** | Task 5: "prints each file it rewrote and exits 0 after a successful sync. `--check` writes nothing and exits 1 when any file would change, for CI." Task 9 step 2 (run, then run again with no change) now succeeds through `make docs-sync`. This matches the declined-enhancement line "beyond 0 and 1". |
| N3 wording | **DISCHARGED** | List D: "`README.md:34-37` (the Gate API "planned and is not available yet")". List U: "**Every `docs sync` naming the operation** (both capability Contracts and Checks lists, `okf-profile.md:55`): reworded to "the sync operation (`Workplan::sync`)"". |
| F1–F8, A1, A2 | **DISCHARGED** (unchanged) | No revision 3 text reopens them. A1 now reads "Every edit keeps its document's line ending (CRLF, LF and mixed fixtures, G13)", which still has a fixture. |

### Review of the new revision 3 text

- **G13 (line endings) and the U entry "with `\n` line endings" becoming "in the document's line ending":** these agree with each other and with the R14 row (CRLF, LF and mixed fixtures). The inventory has `mixed-line-endings` with a fixture. Delivering the reworded Contract sentence is covered by a check. No finding.
- **U additions:**
  - The Binding-resolution gap sentence now hands the obligation to reconcile by link, not by claim.
  - The Checks list gains the issue-reference rule, which G5 builds with its fixtures.

  Both rewordings describe built behaviour. The change records' `affects` lists are widened to the edited sections. No finding.
- **Task 8:** the preview fixture now covers a change listing two capabilities, with a block from another change left unchanged. It is falsifiable on `Workplan::preview`. No finding.
- **Task 9 commits:** commit 1 holds only item outcomes, while the capabilities are still `designed` and listed by open records. Commit 2 delivers and deletes the records. The hook's `docs_contract` run on the staged index is consistent at each commit. No finding.

No new findings. Every issue requirement, and every Contract, Usage, Constraints and Verification statement of the two capability documents, maps to a task with a falsifiable check on a named shipped entry point. The entry points are `Workplan::{load, check, graph, sync, write, status, preview}`, `Gate::workspace(..).assert_clean()`, `raf_workplan::plain::check`, `examples/docs_sync.rs` through `make docs-sync`, and `tests/docs_contract.rs` on this repository. Each task has its check, review, commit, push and issue update, and Task 10 creates the PR.

VERDICT: PASS
