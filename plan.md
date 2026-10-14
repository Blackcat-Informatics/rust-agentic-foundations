# Plan: raf-workplan document model and generated docs (revision 3.3)

- **Worktree:** `.worktrees/7-raf-workplan-model-documents-check`
- **Branch:** `paudley/7-raf-workplan-model-documents-check`
- **Base:** `origin/main` at 762fa99 (merged raf-okf and raf-source)
- **Ledgers:** `issue-analysis.md` (R1–R32, 0 quote misses, 0 comments) and `prior-art-assessment.md`

Revision 2 integrates the five lens reviews in `reviews/plan-{completeness,constraints,design,enhancement,adversary}.md`. Every blocking finding is integrated, and each enhancement is folded in or declined below. Revision 3 integrates the rechecks: completeness N1–N3, adversary F1, F3 and N1–N2, constraints N1–N5 and design N1–N4. It also records the owner's decision on the `cap:source-index` citation (G20).

## Issue summary

Build the `raf-workplan` crate. It loads a repository's capability, change and architecture documents and its source index into **one semantic graph**. On that graph:

- `check` is a set of integrity constraints;
- the generated blocks, `status` and `preview` are projections;
- `Gate::workspace(..).assert_clean()` runs the checks for dev-dependency use.

This PR delivers `cap:workplan-document-model` and `cap:workplan-generated-docs`. The two capability documents and the profile (`docs/architecture/okf-profile.md`) are the binding spec in every section. The plan builds every statement in their Contract, Usage, Examples, Constraints and Verification sections, or names the owning capability for a statement that is not this PR's (Task 9, list U).

## Baseline defaults and standing goals

- The toolchain is the Stage-pinned dated nightly. Toolchains are never changed. The local `make check` stops at `cross-check`, so `make licenses issue-refs` are run directly and hosted CI's cross-check job is that gate's evidence. Nobody claims a local `make check` pass.
- MSRV 1.98.1 via `rust-validate`. Workspace lints are inherited.
- Fail hard: no fallbacks, no first-wins, no silent degradation.
- No issue or PR numbers and no process wording in committed files. Test tokens that would match them are built with `concat!`.
- No attribution. SPDX on every file. Hooks are never bypassed.
- No new dependencies beyond the workspace's (`std::env::temp_dir()` for disk fixtures).
- SPDX tag text inside Rust sources (the `index.md` header renderer and fixtures) is wrapped in `REUSE-IgnoreStart`/`REUSE-IgnoreEnd` or built with `concat!`, so `reuse lint` does not read it as the file's own licence.

## Governing decisions

| # | Decision | Evidence and review source |
|---|---|---|
| G1 | **One semantic graph is the model.** `Workplan::load` builds a graph whose nodes are capabilities, sections, changes, proposals, code symbols, tests and examples. Its edges are bindings, depends, affects and links; **link edges run from section to section**, from the section holding the link to the target section, or the document when there is no anchor. Rules, projections and queries all read the graph; no module re-reads documents. | enhancement 1, 3; document-model Contract "The graph" |
| G2 | **One finding vocabulary.** `Finding { kind: &'static str (stable kebab), location: raf_okf::Location (file plus key or section or line), message, fix }`. `Finding::KINDS` lists every kind, and raf-okf violations map into it with their own locations. **Each rule has its own kind:** every `raf_okf::Rule` variant maps to its own kind `okf-<rule>`. raf-okf gains a public `Rule::ALL` list, with a test in raf-okf asserting it names every variant through an exhaustive match inside the crate. The raf-workplan coverage test iterates `Rule::ALL`, so a variant added later fails the test until it is mapped and has a fixture. Every raf-okf parse-error kind also maps to its own kind, and every raf-workplan rule in the inventory has its own kind. Findings are built only through `const` members of `KINDS`, so no ad-hoc kind string exists. A data-driven coverage test asserts both directions: every kind in `KINDS` is produced by at least one named fixture, and every kind any fixture produces is in `KINDS`. This works despite `#[non_exhaustive]` enums. | enhancement 2; adversary 3; design A |
| G3 | **No duplicated rules.** raf-okf already enforces the single-document rules: schemas, closed keys, id equal to file name, `affects` slug in `changes`, outcome set, section order, PROPOSED in Architecture, marker grammar. `raf-workplan` surfaces those findings and implements only the cross-document and source rules. | design A |
| G4 | **`docs_contract` becomes a thin caller.** It builds a `MemoryTree` from the **git-tracked paths** (`tracked_paths()`, as today), then calls `Workplan::load(..)` and `check()`. That keeps the link oracle on the git index, so a link to an untracked file still fails, and a tracked symlink is still a violation. `docs_contract` keeps only this repository's own policy: the `INVENTORY` table, the doctest-pairs tests (the Usage/Examples blocks of `okf-profile.md` and `source-index.md` equal the crate doctests) and the tracked-symlink rule. It also keeps the checks on tracked Markdown **outside** the document folders (README, AGENTS.md and so on): links, placeholder tokens, a PROPOSED block outside a capability, generated-marker errors and split links. These run through one public raf-workplan function, `raf_workplan::plain::check(path, text, tracked)`, which shares the rule code with `check`, so they are not dropped and not duplicated. The existing tests for them are marked "kept" in the disposition table. Task 7 has a disposition table for every existing test: moved (new name and location), kept, or replaced by a raf-workplan rule. | constraints B2; design B3; completeness F4; adversary 4 |
| G5 | **One issue-reference pattern set.** The matcher in `tests/support/mod.rs`, used by `repo_gates` and `docs_contract`, moves into `raf_workplan::refs` as a public function. `tests/support` calls it, `raf-workplan` uses it for its loaded documents (R25), and AGENTS.md is updated to name the new single home. Because it is public, the issue-reference rule is part of every consumer's `check`, and the delivered document-model Contract states this. | constraints B3; design A |
| G6 | **Sync returns texts; callers write.** `Workplan::sync() -> SyncReport { edits: Vec<(RelPath, String)> }` computes the full new text of every stale or missing file before anything is written. `check`'s staleness comes from the same renderer, so they cannot disagree. The `Tree` seam stays read-only, and `cap:source-index` is unchanged. Writing is done by `Gate`/`Workplan::write(edits, root)`: each target must be a regular file already read through `DiskTree`, or the new `docs/capabilities/index.md`. Writes go to a temporary file and are renamed into place, and nothing is written if any edit fails to compute. | completeness F1; constraints B4; design B1; enhancement 11 |
| G7 | **The source workspace is required.** `Workplan::load(docs_tree, &raf_source::Workspace)`. Fixtures pass a small `MemoryTree` workspace, as `docs_contract`'s `demo_workspace` does. `Gate::workspace(dir)` loads the workspace with roots `["."]` by default and offers `.source_roots(&[..])`. For this repository the roots are `["."]`. Discovery from `.` reaches every package below it, including `fuzz/`'s `raf-source-fuzz` (`discovery.rs:88-154`), so Task 7 asserts the loaded package set is exactly `rust-agentic-foundations`, `raf-okf`, `raf-source`, `raf-workplan` and `raf-source-fuzz`. Loading the fuzz package is harmless: bindings resolving into it are still subject to the role rules. | design B2; enhancement 7 |
| G8 | **No false green.** `Gate::workspace(dir)` is an error when `dir/docs/capabilities` does not exist or holds no capability document. A missing `work/changes` is an empty set of changes, not an error (every capability is then `delivered` or reported by its state rule). `docs_contract` also asserts at least one document of each type and that the known symbols resolve, as today. | enhancement 13 |
| G9 | **Binding roles from index facts:** `code` is a shipping item; `tests` is a `#[test]` function; `examples` is an example target's crate root (`<pkg>::example_crate::<name>`) or a documented item (`documented == true`, not test code) in the library target when its `doctest` is true (cargo ignores the key on other targets). Proving that a doctest runs belongs to `cap:workplan-complete-gate`. | design B4; enhancement 15 (declined in favour of index facts); adversary |
| G10 | **The generated content is specified exactly**, and tested against exact expected bytes per kind. | completeness F5; adversary 6; enhancement 16 |
| G11 | **Generated-block placement:** a `verification-matrix` or `relationships` block is valid only in a Capability document, inside a section. A `catalogue` block is valid only in `docs/capabilities/index.md`. A generated block does not make a section non-empty. Each rule gets a fixture. | completeness F8 |
| G12 | **`index.md`:** the first sync writes the two SPDX HTML-comment lines (copyright and licence text from `SyncOptions`, refused when unset), a blank line and the `catalogue` block. Any non-blank line outside a block other than the SPDX comment is a finding (`index-hand-written`). | enhancement 12; constraints A3 |
| G13 | **CRLF:** the profile accepts CRLF documents. Sync writes each generated block in the document's own line ending, detected from the first line break of the document's **body** (not the frontmatter fence), so a CRLF frontmatter over an LF body gets LF blocks and stays clean; so a CRLF document stays all-CRLF and an LF document all-LF. The output is still a pure function of the input. The generated-docs Contract sentence "with `\n` line endings" is reworded in list U to "in the document's line ending". A document with mixed line endings is refused by `sync` and reported by `check` (`mixed-line-endings`). Fixtures: CRLF, LF, mixed, and a CRLF frontmatter over an LF body (LF blocks, clean). | enhancement 17; adversary N2 |
| G14 | **`preview` is in scope.** The generated-docs Usage and Constraints describe `docs preview <slug>`, printed and never written, and the CLI spec keys it by change slug. `Workplan::preview(change) -> Vec<(CapabilityId, String)>` returns each capability the change lists as it reads once that change's PROPOSED blocks are reconciled: each block replaced by its text without the marker and quote prefix. It returns text and writes nothing. The CLI command arrives with `cap:workplan-cli` and calls it directly. | adversary 1, pass-2 note; completeness F3; enhancement 4 (preview half) |
| G15 | Both capabilities reach **`delivered`** in Task 9. Every statement in every section of both documents is either built or listed in U with its owning capability and exact rewording. Any other unbuilt sentence blocks delivery. | adversary 1; constraints B1 |
| G16 | **The repository's docs pass `check` outright** (R28). R29's "obligations output" belongs to `cap:workplan-reconcile`. | ledger NJ11 |
| G17 | The two dependency issues are closed and merged, the planning essay's binding statement is in the capability documents, and the outcome set is closed. | ledger NJ7–9 |
| G18 | **`status` and `catalogue` are one projection:** `Roadmap`, computed once from the graph. `status` groups planned (`considered` and `designed`), implementing (with its gaps) and delivered; each entry carries its state, open changes and gaps. The catalogue block renders the same `Roadmap` rows in four state groups. A fixture asserts the catalogue rows equal the roadmap rows. | enhancement 5; design A; ledger NJ3 |
| G19 | Real-repository demonstration: `Gate` is run on this repository (Tasks 7, 9). The katamari and purrdf checkouts have no `docs/capabilities` in this format, so the precedent line is dropped instead of being given a no-op task. | completeness A2; adversary 8 |
| G20 | **The owner's decision (verbatim): "Citation fix, no reopen (Recommended)".** The `cap:source-index` Verification sentence that cites binding fixtures in the document-contract test is corrected to cite the raf-workplan fixtures. This is a citation correction with no contract effect, so the capability stays `delivered` and no change record is opened. The PR body states this reading. | constraints N3; user decision |
| G21 | **The owner's decision (verbatim): "Refuse mixed endings (Recommended)".** A document whose body mixes CRLF and LF is an error: `check` reports `mixed-line-endings` and `sync` refuses the file. The delivered Contract states it (list U). This follows AGENTS.md: a malformed or ambiguous document is an error. | adversary pass 3 N2; user decision |
| G22 | **The owner's decision (verbatim): "Additive constant, no reopen (Recommended)".** raf-okf gains a public `Rule::ALL` (empty-payload form, the `State::ALL` / `DocType::ALL` idiom) with an in-crate exhaustive test. It is an additive API with no contract effect, so `cap:okf-profile` stays `delivered`; the PR states this reading. | final review B1; user decision |
| G23 | **The owner's decision (verbatim): "Missing index is a finding (Recommended)".** A missing `docs/capabilities/index.md` is the finding `index-missing`; `sync` creates it, and `check`, the Gate and `docs_sync --check` agree. The index is created in Task 7, so `docs_contract` stays green until Task 9. | final review B3; user decision |

### Enhancements declined

- **`impact` (reconcile)** belongs to `cap:workplan-reconcile`, a designed capability with its own change record. Every query it composes is delivered here (G1, R12), so only its output contract remains there.
- **The two-way coverage queries** belong to `cap:workplan-two-way-coverage`, which is `considered`: its open questions on what counts as public surface and on the exclusion file are undecided. Fixing a query's semantics now would pre-empt that design.
- **Generalising the doctest-pairing rule:** repository policy that the completion gate's example rules supersede. It stays in `docs_contract`.
- **A `docs_sync` example exit-code contract beyond 0 and 1:** kept to what is listed under G6/Task 5.

## Prior art and recurrence

- **Prior attempts:** none.
- **Precedent:** `tests/docs_contract.rs`, about 3,500 lines of ad-hoc rules and about 120 negative fixtures. Its `fixtures::valid` builder and its `assert_hit`/`assert_cases` helpers are ported into `crates/raf-workplan/tests/support` over `MemoryTree`, and each ported test keeps its name (enhancement 6).
- **Recurrence:** defect classes in the last 200 commits are `none` ×2.
- **Lessons:**
  - every Contract claim needs a test or a listed owner;
  - demonstrate on real input;
  - gates must not weaken while code moves.

## Rule inventory

**Cross-document and source rules implemented by raf-workplan**, each a `Finding` kind with a fixture beside a valid neighbour:

| Kind | Rule |
|---|---|
| `depends-missing`, `changes-missing`, `moved-missing` | `depends` or `changes` names a missing capability; a `moved:change:<slug>` outcome names a missing change. Each has its own fixture. |
| `depends-cycle` | self, 2-node and 3-node cycles |
| `wrong-directory` and `nested-directory` | flat directories; a typed document in the wrong directory |
| `stray-markdown` | a non-index Markdown file without frontmatter in a document directory |
| `proposed-missing-change` | a PROPOSED block names a missing change |
| `proposed-change-not-listing` | the change does not list the capability |
| `proposed-section-not-affected` | a PROPOSED block's section is not in the change's `affects` |
| `state-considered-no-proposal`, `state-designed-proposal`, `state-delivered-open-record`, `state-delivered-proposal` | One kind per cross-document state rule, each with a rejected fixture. Every state also has an accepted fixture. The single-document state rules come from raf-okf per G3: `okf-section-empty` (a delivered empty section; comments and generated blocks do not count, G11), `okf-binding-required` (a delivered capability without code or test bindings) and `okf-binding-not-empty` (designed with bindings). Each surfaces through `check` in its own fixture. |
| `no-open-record` | a non-delivered capability with no open change listing it |
| `link-broken`, `anchor-broken`, `link-outside`, `link-symlink` | over every link form raf-okf reads, ported fixtures; anchors follow GitHub rules. `link-symlink` is exercised by the R23 disk fixture, since `MemoryTree` has no symlinks |
| `issue-reference` | via G5 |
| `binding-unresolved`, `binding-wrong-role`, `binding-macro-generated` | as today's `docs_contract` fixtures, plus the examples role of G9 |
| `index-missing`, `generated-stale`, `generated-unknown-kind`, `generated-misplaced`, `index-hand-written`, `mixed-line-endings` | G6, G10–G13 |

**Single-document rules surfaced from raf-okf** (G3): one end-to-end fixture per raf-okf violation kind shows it in `check`, with its location.

## Generated content (G10)

- **`verification-matrix`:**
  - Columns: Item | Kind | Section.
  - One row per entry of `bindings.tests` (Kind `test`) and `bindings.examples` (Kind `example target` or `doctest`), sorted by path.
  - Section rule: a binding substantiates every section other than Verification whose text, outside fences and PROPOSED blocks, contains the bound path as a whole path in a code span, read from the raw section text by byte span. When there is none it substantiates Verification. Several sections are listed in document order, as links to the section anchors.
- **`relationships`:**
  - Columns: Relation | Capability.
  - Rows: Depends on, Depended on by, and Changed by (open changes), each linked, sorted by slug, with relation rows in that order.
  - A relation with nothing is omitted. When every relation is empty the block holds the header only.
- **`catalogue`:**
  - One `## <State>` table per state, in the order delivered, implementing, designed, considered, with empty groups omitted.
  - Columns: Capability (a link to the document) | Description | Gaps. Gaps are filled for implementing rows only: empty lists and unresolved bindings.
  - Rows are sorted by slug.

Each kind gets an exact-output fixture asserting the bytes.

## Completeness contract

| R-id | Requirement | Task | Acceptance check |
|---|---|---|---|
| R1–R3 | plan, implement, reconcile; one current document | 3, 9 | The state and PROPOSED rules in the inventory, each accepted and rejected. The delivered docs pass `check`. |
| R4, R30 | both delivered; docs reconciled | 9 | Both are `delivered` with resolving bindings and exact synced blocks. Every statement in every section is built or in list U. The change records' outcomes are committed before the records are deleted. Every file in list D is updated. `check` passes on this repository. |
| R5–R10, R17 | schemas, sections, change keys, outcomes, no `deferred` | 1 | The end-to-end raf-okf fixtures (G3), and `outcome: deferred` refused. |
| R11, R21 | PROPOSED blocks | 3 | The `proposed-*` fixtures, plus a marker in a code fence that is not read. |
| R12 | graph nodes, edges, four queries | 2 | Fixtures for each node and edge kind, and for the queries: `code_of(cap)`, `capabilities_of(symbol)`, `tests_for(cap, section)` (by the G10 section rule), `sections_of(change)`, and `linking_sections(section)` (section-granular, G1). |
| R13, R19 | bindings | 4 | Resolving: a symbol, a re-export, a test, an example target and a doctest item. Failing: a rename, a missing test, a missing example target, a non-shipping code item, a non-test tests item, an example that is not an example target and not a documented library item, and a macro-generated symbol. Each failure names the capability, the list and the binding. |
| R14, R15, R20, R27 | generated blocks, sync, staleness, determinism | 5 | The exact-output fixture per kind. Sync twice: the second has no edits and the bytes are equal. Per kind, an edited, removed or added row makes `check` report `generated-stale` with file and kind, and sync repairs it. Marker errors. Placement (G11). A file without blocks is untouched. Every edit keeps its document's body line ending (CRLF, LF, mixed and CRLF-frontmatter-over-LF-body fixtures, G13). `index.md` (G12). Running `make docs-sync` twice on this repository writes nothing the second time. |
| R16 | `status` | 6 | Fixture with one capability per state and an implementing capability with an empty and an unresolved binding. The roadmap equals the catalogue rows (G18), and the run on this repository lists all capabilities. |
| R18 | links | 3 | The ported link fixtures over every form, the anchors, `link-outside` and `link-symlink`. `docs_contract` checks the tracked Markdown outside `docs/` and `work/` with the public link checker (`README.md`, `AGENTS.md`, `CONTRIBUTING.md`, `docs/BRAND.md`, …), with a fixture for each. |
| R22 | delivered rules | 3 | The `state-*` fixtures for all four states. |
| R23 | Gate | 7 | Disk fixtures under `std::env::temp_dir()`: a clean repository passes. A broken one panics listing every finding with its location and fix. A missing or empty `docs/capabilities` is an error (G8). `docs_contract` runs the same rules on the tracked tree. |
| R24, R25 | slug ids; issue references | 1, 3 | The raf-okf id fixtures surfaced, and the G5 matcher fixtures (built with `concat!`) through `check`, beside a hex colour and prose numbers. |
| R26 | a fixture per failure | 1–5 | The G2 coverage test over `Finding::KINDS`. |
| R28 | this repository passes | 7, 9 | `docs_contract` (tracked tree) and `Gate::workspace` on the checkout both pass, including in the hook snapshot. |
| R29 | obligations | — | Not used (G16). |
| R31–R32 | dependencies | — | G17. |
| Generated-docs Usage and Constraints | sync, preview, placement, idempotence | 5, 8 | `Workplan::preview(change)` fixture: a PROPOSED block becomes the reconciled text and nothing is written. Placement (G11) and idempotence as above. |

## Task structure

Every task follows the same pattern:
1. Implement, with a sonnet/medium implementer given the exact scope and the NO-DEFERRALS clause.
2. Run the task's **Check**.
3. An independent reviewer writes `$STAGE/tasks/TN-review.md` with `VERDICT: PASS` or `BLOCKED`.
4. Fix and recheck.
5. Commit with explicit files and hooks on, then push.
6. Post the task summary to the issue.

The per-task gate is `cargo test -p raf-workplan --locked`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --locked --test docs_contract --test repo_gates` and `make reuse`.

## Task 1: crate, findings and document loading

- **Scope:**
  - raf-okf: add `Rule::ALL` with its exhaustive in-crate test (G22).
  - `crates/raf-workplan` (Cargo.toml, `lib.rs`, `finding.rs`, `load.rs`, `model.rs`) and the workspace member.
  - `Workplan::load(docs: &dyn Tree, source: &Workspace)` reads `docs/capabilities/`, `docs/architecture/` and `work/changes/` through the tree, and validates each document with raf-okf into typed values.
  - G2 findings, the directory rules, and the ported test-support builder.
  - The CHANGELOG entry.
- **Check:** the end-to-end raf-okf fixtures, the directory fixtures, and the G2 coverage test (initially over the Task 1 kinds).
- **Commit:** `feat(raf-workplan): load documents into typed findings`.

## Task 2: the graph and its queries

- **Scope:** G1 nodes and edges, including section-granular links; the reference and cycle rules; and the R12 queries.
- **Check:** the graph and query fixtures, plus `depends-missing`, `changes-missing`, `moved-missing` and `depends-cycle` (an `affects` section that does not exist is raf-okf's `BadAffectsSection` and `SectionMissing`, surfaced per G3).
- **Commit:** `feat(raf-workplan): document graph with section-granular links`.

## Task 3: proposal, state, link and reference rules

- **Scope:**
  - The `proposed-*`, `state-*`, `no-open-record`, link and anchor rules, with the public link checker.
  - G5: the move of the issue-reference matcher into `raf_workplan::refs`, with `tests/support` calling it and AGENTS.md updated.
- **Check:** the fixtures in the inventory. `cargo test --locked --test repo_gates` still passes with the moved matcher, and `make issue-refs` too.
- **Commit:** `feat(raf-workplan): proposal, state, link and reference rules`.

## Task 4: bindings

- **Scope:** G9 roles, with findings that name the capability, the list and the binding.
- **Check:** the R13 fixtures.
- **Commit:** `feat(raf-workplan): resolve bindings through the source index`.

## Task 5: generated blocks, sync and write (after Task 6)

- **Scope:**
  - The G10 renderers.
  - Staleness from the same renderer.
  - `Workplan::sync()` and `write()` (G6).
  - G11–G13.
  - `examples/docs_sync.rs`: takes an optional root (default `.`), prints each file it rewrote and exits 0 after a successful sync. `--check` writes nothing and exits 1 when any file would change, for CI.
  - `make docs-sync`.
- **Check:** the R14/R15/R20/R27 fixtures. `cargo run -p raf-workplan --example docs_sync -- <tempdir fixture>` writes the expected files once and nothing on a second run.
- **Commit:** `feat(raf-workplan): generated blocks and deterministic sync`.

## Task 6: roadmap: `status` and the catalogue

- **Scope:** `Roadmap` (G18). `status()` returns it. **Task 6 is implemented before Task 5**, so the catalogue renderer in Task 5 consumes it.
- **Check:** the R16 fixtures, and roadmap rows equal catalogue rows.
- **Commit:** `feat(raf-workplan): roadmap for status and the catalogue`.

## Task 7: the Gate, and docs_contract on raf-workplan

- The first `make docs-sync` run creates `docs/capabilities/index.md` here (G23), with the SPDX text taken from `SyncOptions` fields that the `docs_sync` example fills from the repository's own header (copyright and licence) via explicit arguments.

- **Scope:**
  - `Gate::workspace(dir)` with `.source_roots()`, `check()` and `assert_clean()` (G7, G8).
  - `tests/docs_contract.rs` reduced per G4. **Disposition table:** every existing test function is listed in `$STAGE/tasks/T7-dispositions.md` (under the git-ignored stage directory) as moved (new path::name), kept, or replaced by a raf-workplan rule (kind). No test is dropped without a row, and the reviewer checks that the table covers every `#[test]` in the old file.
- **Check:**
  - The Gate disk fixtures.
  - `cargo test --locked --test docs_contract --test repo_gates` passes on this repository and in the hook snapshot.
  - The count of model-rule fixtures (moved and new) is at least the count of model-rule tests removed from `docs_contract`.
- **Commit:** `test(docs-contract): check the repository through raf-workplan`.

## Task 8: `preview`

- **Scope:** G14.
- **Check:** a preview fixture over a change listing two capabilities, one with two PROPOSED blocks from it and another block from a different change that stays as is. The output matches the expected reconciled text, and the tree is unchanged.
- **Commit:** `feat(raf-workplan): preview a capability as its changes would leave it`.

## Task 9: deliver the two capabilities

**Steps:**
1. Add the empty `verification-matrix` and `relationships` blocks to the two documents.
2. Run `make docs-sync`, which fills every block (the index exists since Task 7). Run it again: no change.
3. Set both documents to `delivered`, with code, test and example bindings to the built items.
4. Reconcile every section.
5. Commit the change records with their outcomes filled in.
6. In a following commit, delete the change records (completion).

**U, the unbuilt statements and their owners**, reworded exactly as below:
- **document-model Usage:** "The `cargo workplan` binary exposes the same checks and the change lifecycle" becomes "The command line, a separate capability ([command line](./workplan-cli.md)), runs the same checks."
- **generated-docs Usage:** the `cargo workplan docs sync` / `check` / `docs preview` block is replaced by the library usage (`Workplan::sync`, `write`, `preview`, the `docs_sync` example and `make docs-sync`). The command line is mentioned only by link, as above.
- **generated-docs Constraints:** "Preview output is printed and never written to a file" becomes "`preview` returns text and writes nothing."
- **generated-docs Constraints, preview limit (Task 8 finding):** add "`preview` refuses a PROPOSED block whose quoted text holds a fenced code block, because the block reader ends a block at the fence; the error names the line." Lifting the limit would change the block-end rule of `cap:okf-profile` and is not part of this issue.
- **generated-docs Contract, "Mechanical versus explanatory":** "reference sections, relationship tables, status summaries, links, indices and verification matrices" becomes the three built kinds.
- **document-model Explanation:** the `binding-missing` / `example-failing` obligations sentence becomes a link stating that the obligations are defined by [reconcile](./workplan-reconcile.md).
- **document-model Contract, Binding resolution (lines 104-106):** "`status`, the catalogue and the reconcile obligations report each one as a gap" becomes "`status` and the catalogue report each one as a gap; [reconcile](./workplan-reconcile.md) defines the obligation it becomes."
- **Every `docs sync` naming the operation** (both capability Contracts and Checks lists, `okf-profile.md:55`): reworded to "the sync operation (`Workplan::sync`)", so no reader takes it for a built command.
- **generated-docs Contract, Sync:** "with `\n` line endings" becomes "in the document's line ending" (G13).
- **document-model Contract, Checks:** gains the issue-reference rule as part of every consumer's `check` (G5), and a bullet stating that a document whose body mixes CRLF and LF line endings is an error (G21).
- **generated-docs Contract, Sync:** gains "A document whose body mixes line endings is an error, and sync refuses it." (G21)
- **document-model Constraints:** the first bullet is widened from "this model adds rules between documents and the source" to "this model adds rules between documents and the source, and the line-ending rule for the documents it rewrites" (G21).

**D, other committed files made stale by this PR**, each corrected:
- `docs/capabilities/source-index.md:1488-1500` (the Verification text citing binding fixtures in the document-contract test now cites the `raf-workplan` fixtures);
- `docs/architecture/okf-profile.md:31-40, 173-174`;
- `AGENTS.md:41-45, 49-56, 63, 74, 133-134, 208-223`;
- `README.md:31`;
- `CONTRIBUTING.md:32-36`;
- the `tests/docs_contract.rs` module doc (4-19) and comments (61-62);
- `docs/architecture/okf-profile.md:55`;
- `README.md:34-37` (the Gate API "planned and is not available yet");
- `crates/raf-okf/tests/usage_docs.rs:4-6`;
- the `INVENTORY` table in `tests/docs_contract.rs` (state changes and removed change records; its `index.md` row is added in Task 7, with the commit that creates the file).

The change records' `affects` lists are widened to the sections the reconcile actually edits (Usage, Constraints, Explanation).

**Check:**
- `Gate::workspace(repo)` on the checkout, and `docs_contract` on the tracked tree.
- `make docs-sync` twice with no second change.
- `make -k check` (all gates except cross-check), then `make licenses issue-refs`.
- Hosted CI on the pushed head is green on all jobs, including cross-check.

**Commits:**
1. `docs(changes): record the outcomes of the workplan model and generated docs`: only the item outcomes in the two change records.
2. `docs(capabilities): deliver the workplan document model and generated docs`: everything else in this task: the states, bindings, synced blocks, `index.md`, the reconciled sections, list D, the `INVENTORY` edit, and the deletion of both records.

## Task 10: PR, plan publication, handoff

- Run an artifact inventory (haiku) and a completeness review of the implementation against this plan.
- `pr-create` with `Closes`, the changes, the actual validation (local gates, hosted CI) and residual risks.
- Post the plan to the issue and the PR, and post the least-confident and should-know note.
