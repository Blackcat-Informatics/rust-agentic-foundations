# Completion audit: raf-workplan document model and generated docs

- Worktree: `.worktrees/7-raf-workplan-model-documents-check`, HEAD `a3d60dd`, compared with `origin/main` (`762fa99`, the merge base).
- Inputs read: `issue.md`, `issue-analysis.md` (R1-R32), `requirements.json` (B1-B16), `plan.md` (rev 3.3, G1-G23, tasks 1-10), `tasks/T1..T8-review.md` (every task ends `VERDICT: PASS`; no T9 review file, its evidence is in `t9-*.log`).
- No tracked file was modified. The worktree stayed clean (`git status --porcelain` empty) after every command, including both `make docs-sync` runs. Experiments ran in a separate clone at `/tmp/raf7audit/copy`, which was reset after each one. Harness: `/tmp/raf7audit/scripts/exp.sh`. Logs: `/tmp/raf7audit/exp/*.log`.
- `CARGO_BUILD_JOBS=6`. The toolchain was not changed.

## Entry points run

| Command | Result |
|---|---|
| `cargo test -p raf-workplan --locked` | exit 0. 18 targets (5 unit, 17 integration test files, 6 doctests): 284 passed, 0 failed |
| `cargo test --locked --test docs_contract --test repo_gates` | exit 0. docs_contract 26 passed, repo_gates 57 passed |
| `Gate::workspace(<this repo>).assert_clean()` (the test `docs_contract::repository_documents_satisfy_the_document_model`, `tests/docs_contract.rs:509-514`, whose root is `gate_root()`) | ok, no finding |
| Same binary with `RAF_GATE_ROOT=/tmp/raf7audit/copy` (a clean clone) | ok, the baseline for the experiments |
| `cargo run -p raf-workplan --example docs_sync -- --check` (the documented command, run from the repository root) | exit 0, prints nothing |
| `make docs-sync`, run twice | both exit 0, no `rewrote` line; `git status` empty; sha256 of `docs/capabilities/*.md` identical before and after (`398c4938...`) |
| Real regeneration in the copy: every generated block emptied and `index.md` deleted, then `make docs-sync ROOT=<copy>` twice | 1st run: `rewrote` index.md, workplan-document-model.md and workplan-generated-docs.md, and every real block plus the index is byte-identical to HEAD. 2nd run: no output. `--check` afterwards: exit 0. The only remaining diff is the two fenced examples my perl edit emptied, which sync correctly does not touch |
| `make issue-refs` | exit 0 (57 passed) |
| `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `RUSTDOCFLAGS=-Dwarnings cargo doc --no-deps -p raf-workplan -p raf-okf` | all exit 0 |
| `make -k check` (`t9-gates.log`, `t9-check.log`) | every local gate passes up to `cross-check`, which stops because the local machine lacks the cross targets. Per the plan, hosted CI is that gate's only evidence |
| Hosted CI | **not run yet.** CI triggers on `pull_request` and on pushes to `main`. No PR exists for the branch (`gh pr list --head` gives `[]`) |

## Break-a-doc experiments (copy, through `Gate::workspace(..).assert_clean()` and `docs_sync --check`)

Each experiment must produce a named finding.

| Experiment | Gate finding(s) | docs_sync --check | Result |
|---|---|---|---|
| Stale block: one matrix row deleted in workplan-generated-docs.md | `generated-stale docs/capabilities/workplan-generated-docs.md:273` (names the kind `verification-matrix`) | `stale ...`, exit 1 | PASS |
| Missing binding: `Workplan::write` changed to `write_gone` | `binding-unresolved ... (key bindings)`, naming `cap:workplan-generated-docs`, the `code` list and the path | exit 0 (sync is unaffected) | PASS |
| PROPOSED block naming a missing change, in a delivered capability | `proposed-missing-change ...:77` and `state-delivered-proposal` | exit 0 | PASS |
| Malformed PROPOSED marker (`change:Bad_Slug`) | `okf-parse-malformed-proposed ...:77`, plus cascading `generated-stale` on the relationships block and the catalogue because the document drops out of the model | exit 2, "fix the findings of `check` for it first" | PASS |
| `depends` cycle (document-model and generated-docs) | `depends-cycle ... (key depends)`, naming both members, plus `generated-stale` on both relationship tables | `stale` x2, exit 1 | PASS |
| Delivered capability with an empty Explanation | `okf-section-empty ... (section Explanation)`, plus cascading stale blocks | exit 2 | PASS |
| An issue-number token appended to a capability (built with printf) | `issue-reference ...workplan-reconcile.md:187` | exit 0 | PASS |
| Unknown frontmatter key `owner` | `okf-unknown-key ... (key owner)` | exit 2 | PASS |
| Broken link, broken anchor | `link-broken ...:187`, `anchor-broken ...:187` | exit 0 | PASS |
| Link to a file that is on disk but untracked | `link-broken ...` (the link oracle is the git index) | exit 0 | PASS |
| `outcome: deferred` in a change record | `okf-bad-outcome ... (key items)` | exit 2 | PASS |
| PROPOSED block in a designed capability naming an existing change that does not list it | `proposed-change-not-listing ...:27`, `state-designed-proposal` | exit 0 | PASS |

## Requirements and acceptance criteria

| Req | Requirement | Evidence | Verdict |
|---|---|---|---|
| R1-R3 | plan, implement, reconcile; one current document | The state and PROPOSED rules are tested in `tests/states.rs` (16 passing). Both delivered docs pass the Gate. `Workplan::preview` gives the reconciled reading (`tests/preview.rs`, 20 passing) | PASS |
| R4 | both capabilities delivered | `state: delivered` in both documents. Gate is clean. `index.md` lists both under Delivered | PASS |
| R5-R7 | Capability frontmatter and the seven sections | raf-okf rules surface through `check` (`rules::every_rule_has_a_fixture_reported_with_file_and_location`). Experiments: unknown key, empty section | PASS |
| R8-R10 | Change keys and outcomes; no `deferred` | `okf-bad-outcome` on `deferred` (experiment). `moved-missing` fixture (`references.rs`) | PASS |
| R11, R21 | PROPOSED blocks; none for a missing change | `proposed-*` fixtures; experiments 3a and 3b and the not-listing case; the fence-marker fixture | PASS |
| R12 | Graph nodes, edges, queries | `tests/graph.rs` 22 passing (every node and edge kind, section-granular links, five queries) | PASS |
| R13, R19 | Binding resolution through raf-source | `tests/bindings.rs` 23 passing; missing-binding experiment | PASS |
| R14, R15, R20 | Generated blocks; deterministic sync; in-sync check | `tests/generated.rs` 39 and `tests/sync.rs` 17 passing; stale-block experiment; full regeneration in the copy is byte-identical to HEAD | PASS |
| R16 | `status` roadmap | `Workplan::status` gives `Roadmap`; `tests/roadmap.rs` 12 passing, including this repository with every capability once; `sync::the_catalogue_of_this_repository_equals_its_roadmap` | PASS |
| R17, R18, R22 | schema/closed keys, links, delivered rules | experiments (unknown key, broken link and anchor, untracked link, delivered with a PROPOSED block, empty section) | PASS |
| R23 | `Gate::workspace(..).assert_clean()` | `tests/gate.rs` 13 passing; `docs_contract` calls it on this repository; every experiment above goes through it | PASS |
| R24 | IDs are slugs only | `references::r24_*` fixtures pass (behaviour PASS). **Their names carry a ledger id; see REQUIRED 1** | PASS (behaviour) |
| R25 | `check` rejects issue-number references | `issue-reference` experiment; `tests/refs.rs` 11 passing; the matcher is shared with `repo_gates` | PASS |
| R26 | Fixture tests for every `check` failure | `kinds::every_kind_is_produced_by_a_named_fixture_and_every_produced_kind_is_a_kind` and `every_rule_and_every_reader_error_has_its_own_kind` (over `Rule::ALL` and `ErrorKind::ALL`) pass | PASS |
| R27 | `docs sync` is idempotent and byte-deterministic | `make docs-sync` twice on the repository (no write, same hashes); full regeneration twice in the copy; `sync::syncing_twice_...`, `the_report_is_a_pure_function_...` | PASS |
| R28 | This repository passes `check` | Gate clean; docs_contract 26/26; `docs_sync --check` exit 0 | PASS |
| R29 | obligations output | Not used: the repository passes outright (G16) | N/A |
| R30 | Capability docs reconciled | Both documents state built behaviour. List U rewordings are present (`Workplan::sync` naming, preview limit, line-ending rule, reconcile links). List D files are updated (okf-profile architecture page, `source-index.md` citation, AGENTS, README, CONTRIBUTING, docs_contract module doc). Both change records are deleted after their outcome commit (`d65e9f4`, then `a3d60dd`). **Exception: two present-tense sentences about the unbuilt command line; see REQUIRED 2** | FAIL (REQUIRED 2) |
| R31-R32 | dependencies | Both dependencies are closed and merged (base `762fa99`) | PASS |

## Repository rules

- **Built-only claims.** I grepped the delivered docs, the okf-profile pages, README, AGENTS and CONTRIBUTING for `cargo workplan`, `docs sync`, `docs preview`, `obligation`, `reconcile`, `CLI`, `command line`, `planned` and `not available`:
  - Every `docs sync` naming is gone from the delivered docs; the remaining hits are in designed capabilities.
  - Reconcile and obligations appear only as links to `workplan-reconcile.md`.
  - README marks the census gate as "designed and not available yet" and the workplan gate as built.
  - Two sentences still state unbuilt behaviour in the present tense (REQUIRED 2).
- **Issue numbers and process wording.** `make issue-refs` passes. Commit messages carry no issue tokens. A manual grep found the ledger id `R24` in three committed test names, in a comment and in two lines of a delivered document (REQUIRED 1). The `repo_gates` matcher does not catch this form.
- **Attribution.** No credit or co-author text in the 10 commit messages or in the added lines. Every commit has author and committer `Patrick_Audley`.
- **`.deficiencies`.** Only the notice and the marker; no entries.
- **SPDX/REUSE.** 315/315 (`t9-gates.log`).

## Plan task artifacts

| Task | Artifact | Present |
|---|---|---|
| 1 | `Rule::ALL` with an in-crate exhaustive test (`violation.rs:469`, `:692`); the crate, `finding.rs`, `load.rs`, `model.rs`; CHANGELOG entry | yes (commit `79cc6fb`) |
| 2 | `graph.rs`, queries, `references.rs` cycle and missing-reference fixtures | yes (`73c2d71`) |
| 3 | `rules.rs`, `links.rs`, `plain.rs`; `refs.rs` moved out of `tests/support` (`tests/support/mod.rs:29` delegates); `AGENTS.md:141` names the new home | yes (`b6dff17`) |
| 4 | `bindings.rs`, `tests/bindings.rs` | yes (`2992c66`) |
| 6 | `roadmap.rs`, `tests/roadmap.rs` | yes (`6970f8a`) |
| 5 | `generated.rs`, `sync.rs`, `examples/docs_sync.rs`, `make docs-sync` | yes (`df2c46b`) |
| 7 | `gate.rs`, `tests/gate.rs`, the reduced `tests/docs_contract.rs`, `tasks/T7-dispositions.md` (stage) | yes (`45ff252`) |
| 8 | `preview.rs`, `tests/preview.rs` | yes (`d95b369`) |
| 9 | outcomes commit `d65e9f4`, then delivery commit `a3d60dd` (states, bindings, synced blocks, `index.md`, list U and list D, both change records deleted) | yes, except REQUIRED 1 and 2 |
| 10 | PR, plan publication, handoff | not started (no PR yet), which is expected at this point |

`crates/raf-okf/tests/usage_docs.rs:4-6` (list D) is unchanged, and it is still accurate: the doctest-pair tests stay in `docs_contract` (`tests/docs_contract.rs:1256`).

## Facts the PR body must state

1. **G20.** The `cap:source-index` Verification sentences (`docs/capabilities/source-index.md` around lines 1489 and 1497) now cite the `raf-workplan` binding fixtures instead of the document-contract test. This is a citation fix with no contract effect, so the capability is not reopened and no change record is opened. I checked that the cited fixtures exist: `tests/bindings.rs:81` (`demo::body::FenceBlock` refused), `:134` and `:149`.
2. **G22.** The raf-okf API addition is additive and `cap:okf-profile` stays delivered. The owner's decision named `Rule::ALL`, but the delivered addition is wider, so the PR body must list all of it:
   - `Rule::ALL`, `ErrorKind::ALL`, `RuleTag`, `ErrorTag`, `Rule::kind`, `ErrorKind::tag` and `name`;
   - in-crate tests that keep each list equal to its variants.

   `cap:okf-profile` does not mention these items.
3. **`RuleTag` and `ErrorTag` are exhaustive on purpose** (`violation.rs:335-339`, `error.rs:458-462`). A later rule or error variant breaks every exhaustive match downstream, which is a semver-major event for raf-okf. That is the intent: every consumer must map the new kind.
4. **The preview fence limit.** `preview` refuses a PROPOSED block whose quoted text holds a fenced code block, and names the line. The cause is that the block reader of `cap:okf-profile` ends a block at a fence. Lifting the limit would change that capability's block-end rule. It is documented in generated-docs Constraints. A misindented continuation line is refused the same way.
5. **Cross-check evidence comes only from hosted CI.** Locally `make check` stops at `cross-check` (the targets are not installed). Every other gate passed locally: fmt, clippy, test, test-release, doc, deny, reuse, msrv, fuzz smoke, licenses, issue-refs. Nobody may claim a local `make check` pass.
6. **The Gate's one non-git fallback.** When `git` answers that the directory is not inside a work tree (the commit-hook snapshot), every file stands for a tracked path, which weakens the untracked-link rule there. `Gate::tracked_paths` overrides it. This is documented in document-model Constraints and in the `Gate` rustdoc. Every other git failure is an error.
7. **The `docs_sync --check` exit code.** On a repository with no index and no SPDX flags, it exits 2 (an error naming `--copyright`), not 1. This was accepted in the T5 review. `make docs-sync` always passes the flags.
8. **Cascading findings.** A document that fails single-document validation drops out of the model. `check` then also reports `generated-stale` on the tables that list it, and sync refuses until it loads. This is by design and the message says so.

## REQUIRED fixes

1. **Remove the requirement-ledger id from committed files.** A ledger id is development-process wording, and the CLAUDE.md "Never" list forbids it.
   - Rename these tests (for example, drop the `r24_` prefix):
     - `crates/raf-workplan/tests/references.rs:209` `r24_a_slug_id_that_differs_from_the_file_name_is_refused`
     - `:225` `r24_a_slug_that_is_not_kebab_case_is_refused`
     - `:242` `r24_a_matching_kebab_slug_is_accepted`
   - Reword the comment `// R24: ...` at `references.rs:207`.
   - Update the binding at `docs/capabilities/workplan-document-model.md:57` and run `make docs-sync`, so the verification-matrix row (`:440`) follows.
   - Re-run raf-workplan tests, docs_contract, `make issue-refs` and `docs_sync --check`.
2. **Do not state the unbuilt command line as present behaviour in a delivered document.**
   - The two sentences:
     - `docs/capabilities/workplan-document-model.md:266-267`: "The command line, a separate capability ([command line](./workplan-cli.md)), runs the same checks."
     - `docs/capabilities/workplan-generated-docs.md:193-194`: "The command line, a separate capability ([command line](./workplan-cli.md)), calls the same operations."
   - `cap:workplan-cli` is `designed`, and the document's own Constraint forbids a statement about unbuilt behaviour in a delivered contract.
   - Reword both to a fact that is true now, for example "A command line is specified separately ([command line](./workplan-cli.md)) and is not part of this capability."
   - Then re-run the Gate and `docs_sync --check`. The matrix is unaffected unless a bound path is named.
3. **Hosted CI must be green before the work is claimed complete or merged.** Run it on the PR head after both fixes, on every job including `cross-check`. This is a gate, not a code fix; Task 10 must record the run.

VERDICT: BLOCKED
