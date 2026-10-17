# Gap Analysis

**Issue:** #10 (M2: rest of the planning essay)
**PR:** #20 (head 43c7e02, base main)
**Branch:** paudley/10-m2-rest-of-the-planning-essay-entity
**Base:** origin/main (scope `origin/main...HEAD`)
**Files changed:** 204 (42 commits, +37194/-1265)

Method: a detached scratch worktree `/tmp/g10` at 43c7e02 with `CARGO_TARGET_DIR=/tmp/g10-target`,
`CARGO_BUILD_JOBS=6` (removed afterwards). Nothing in the issue worktree was modified, staged, stashed or
reset. No toolchain change. The `cargo-workplan` binary was built once and every probe below ran through it
(`cargo-workplan workplan --root /tmp/g10 ...`). Each scratch edit was restored and `git status` was clean
at the end. I also ran `cargo test -p raf-workplan --test history` (23 passed) and
`cargo test --test docs_contract --test repo_gates` (28 and 57 passed) at HEAD.

## Headline

The five capabilities are built, delivered through `change complete`, and their records are deleted.
Nothing was moved or dropped. The five owner decisions are honoured, and hosted CI is green 12/12 on
43c7e02. Of the 55 Contract, Constraints and Verification claims I sampled against the code and the
running binary, 53 hold.

Two delivered Contract statements are false as built. The count in the completion headline counts
obligations, not sections. A line-broken claim marker silently becomes a plain comment. Both sections
carry `accepted` agent assessments.

One view, `evidence claims`, silently skips a capability that does not load.

The PR body is stale and incomplete. It says hosted CI has not run. It also points to git-ignored stage
notes for the reworded sentences the plan required it to list, and that list misses one delivered
rewording.

## Requirements Coverage

| # | Requirement | Status | Location | Gap Description |
|---|-------------|--------|----------|-----------------|
| R1 | entity history (`--follow`, `-L`) | COMPLETE | `crates/raf-workplan/src/history.rs`, `git/log.rs`, `cargo-workplan/src/history.rs` | Verified live. The argv rows match the Contract exactly (PATH stub). The question order is right, `--since` reaches no process, module bindings list their files, ids are refused before any git call, and JSON and the table agree. Advisory: a completed change has no `history change:<slug>` because its record is deleted, and `history cap:` does not include the change records that shaped the capability. |
| R2 | evidence and claims; invalidation | COMPLETE with gaps | `raf-workplan/src/evidence*.rs`, `obligation/trace.rs`, `lifecycle/changed.rs`, `raf-okf/src/body/marker.rs` | 23 `evidence record` probes all match the Contract. `claim-unsupported`, `evidence-stale` and `claim-evidence-missing` were seen live. Gaps: G2 (line-broken marker is silently a plain comment) and G3 (`evidence claims` silently skips a capability that does not load). |
| R3 | prose assessment; invalidation; gate | COMPLETE with gap | `raf-workplan/src/assessment.rs`, `lifecycle/complete.rs` | Shown live: recording is deterministic, a regenerated block leaves an assessment current, an edit of another section leaves it current, the context is identical for human and agent, `assessment-id-mismatch` works, and condition 6 refuses with no cargo run. Gap G1: the headline count. |
| R4 | two-way coverage | COMPLETE (scope per owner decision) | `raf-workplan/src/coverage.rs`, `raf-source/src/exports.rs`, `work/coverage/default.md` | Twelve record variants each behave as stated. The reports cover the whole workspace (212 code gaps, 439 test gaps: raf-okf, raf-source, root). `--crate raf-workplan` and `--crate cargo-workplan` report no gap. Advisory: a capability that does not load makes `check` report every item it binds as "bound by no capability". |
| R5 | current rationale | COMPLETE | `raf-workplan/src/rationale.rs`, `docs/architecture/*.md` | Probes: an unlinked page, a self-link (does not count), an empty conditions list (a code-span item does not count), `## Decision ##`, a setext heading (does not count) and a link from a capability. All behave as stated. |
| R6 | each reaches `delivered` through `change complete` | COMPLETE | delivery commits d561881/09a2c8d, fcd69d1, 7b8ae50, 68a3342, 023df72 (+7 reopened) | `status` shows delivered 12, implementing 0 and considered 0. The five were `considered` on main. |
| R7 | record deleted; history in Git | COMPLETE | `git log --diff-filter=D` | One deletion commit per record. Entity-history has two: d561881, then 09a2c8d after the reopening. |
| R8 | no deferrals; `moved:` for any cut | COMPLETE for items, PARTIAL for the plan's R8 check | records at `<commit>^` | Every item of all 13 deleted records is `implemented`. The plan's R8 check, "reworded claims are listed in the PR body", is not met. The PR body points to `.stage` notes (git-ignored, invisible to reviewers), and one delivered rewording is in no list (G5). |
| R9 | lifecycle run with the tool | COMPLETE | validation 11F1-11F3, Task 12 | `change reconcile` came back empty and `change complete` printed `completed: removed ...` for each record. |
| R10 | one task set per capability | COMPLETE | - | - |
| R11 | scope-out (M1) | N/A | - | - |
| R12 | depends on #9 | READING RECORDED | plan "Baseline" | #9 is still OPEN. The plan's reading ("nominal, not waited for") is not in the PR body's owner readings (LOW). |
| R13 | premise (documents and records on main) | TRUE | origin/main | - |

## Acceptance Criteria Status

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Each capability reaches `delivered` through `cargo workplan change complete` | MET | validation 11F3 and Task 12. At HEAD each document says `state: delivered` and `status` lists 12 delivered. |
| Its change record is deleted on completion; the history is in Git | MET | Each `--diff-filter=D` commit flips the state and deletes the record together. |
| No deferrals: any scope that leaves an item is `moved:` to a tracked change | MET (items) | No `moved`, no `dropped`. One delivered clause was removed without being listed or tracked (G5). |

## Contract, Constraints and Verification claims sampled against code and binary (55)

Every row was run through the built binary unless it is marked "code".

| # | Capability | Claim | Result |
|---|---|---|---|
| 1-3 | entity-history | Id kinds `cap/change/arch/evidence/assessment`. Another kind (`foo:bar`), `-rf`, `--output=x` and `cap:-rf` exit 2 and no git starts. | holds |
| 4 | entity-history | `--since` matches YYYY-MM-DD from 1970-01-01 to 2099-12-31. Impossible days (2026-02-30, 2026-13-01) are refused. | holds |
| 5 | entity-history | The three git argv forms, exactly. | holds (stub on PATH logged them byte for byte) |
| 6 | entity-history | Question order: version, shallow, document log, spans by binding path, module files by binding then path. | holds |
| 7 | entity-history | `--since` reaches no process. | holds (no argv carries it) |
| 8 | entity-history | A module binding lists the commits of its files (rationale.rs: d6f8804 and c54db19, the only two commits of the file). | holds |
| 9 | entity-history | Newest first. JSON carries the same entries with `kind:"history-entry"`, the full id and the author date. | holds |
| 10 | entity-history | The `<id>` is matched against loaded documents only. A deleted change exits 2. | holds |
| 11 | entity-history | The author-date fixtures. | holds (23/23 history tests) |
| 12-13 | evidence-claims | Exit 0 is `pass`, any other status is `fail`. A value over its limit is `fail`. | holds |
| 14 | evidence-claims | Values are compared on their decimal text, not as floats (`0.10000000000000001` over `0.1` fails; `0.1000...0` passes). | holds |
| 15 | evidence-claims | No exponent: `1e-3` exits 2 and nothing is written. | holds |
| 16 | evidence-claims | `status=ok` is an error that writes nothing. `n=3` is stored. | holds |
| 17 | evidence-claims | A name printed twice exits 2. | holds |
| 18 | evidence-claims | A `--limit` never printed exits 2, for a passing and for a failing command. | holds |
| 19 | evidence-claims | 1025 values exit 2; 1024 record. | holds |
| 20 | evidence-claims | A command that cannot start exits 2 and writes no record. | holds |
| 21 | evidence-claims | `--revision` must be 40 or 64 hex digits. | holds |
| 22 | evidence-claims | A new record never replaces a file; `--update` replaces it. | holds |
| 23 | evidence-claims | stdin closed, the workspace root as cwd, the caller's environment kept, no `LC_ALL` added. | holds (probe printed `stdin=0 cwdroot=1 env=1 lcunset=1`) |
| 24 | evidence-claims | No output text is persisted. | holds |
| 25 | evidence-claims | A duplicate limit, or half an SPDX pair, exits 2. | holds |
| 26 | evidence-claims | `check` reports `claim-evidence-missing` at the marker's file and line. | holds |
| 27 | evidence-claims | `claim-unsupported` is raised for a failing record and again for each over-limit value. | holds (2 lines) |
| 28 | evidence-claims | An edit inside a bound symbol gives `evidence-stale` naming it. | holds |
| 29 | evidence-claims | The views exit 2 for an unknown evidence id or capability. | holds |
| 30 | prose-assessment | Recording the same verdict on the same inputs writes the same bytes. | holds (cmp) |
| 31 | prose-assessment | A regenerated block leaves the assessment current. | holds (edited inside the verification-matrix block) |
| 32 | prose-assessment | An edit of another section leaves it current; an edit of the section stales it. | holds |
| 33 | prose-assessment | The context is identical for human and agent. | holds (cmp) |
| 34 | prose-assessment | A section that is not one of the seven is refused with exit 2. | holds |
| 35 | prose-assessment | A misnamed record is `assessment-id-mismatch`. | holds |
| 36 | complete-gate | Condition 6 refuses, keeps the record and runs no cargo. | holds |
| 37 | complete-gate | "Each section without a current accepted assessment is also an ... obligation ... with the count `sections without a current accepted assessment`" | **FALSE (G1)** |
| 38 | two-way-coverage | A scope entry given as a crate name gives `scope-crate-missing` naming the package; an unknown entry lists the packages. | holds |
| 39 | two-way-coverage | An empty reason gives `okf-empty-reason`. | holds |
| 40 | two-way-coverage | An unmatched exclusion, and a sibling-prefix one, give `coverage-exclusion-unmatched`. | holds |
| 41 | two-way-coverage | A crate-root exclusion (`raf_okf`, `raf_okf::*`) is refused. | holds |
| 42 | two-way-coverage | No record gives exactly one `coverage-record-missing`. | holds |
| 43 | two-way-coverage | `scope: []` enforces no crate, and `scope: [raf-okf]` enforces it. | holds |
| 44 | two-way-coverage | The reports cover the whole workspace; JSON has `in_scope`. | holds |
| 45 | current-rationale | A conditions list whose only item is a code span is `decision-conditions-missing`. | holds |
| 46 | current-rationale | Unlinked page reported; a self-link does not count. | holds |
| 47 | current-rationale | A closing `##` run counts; a setext heading does not. | holds |
| 48 | current-rationale | A link from a capability counts. | holds |
| 49 | okf-profile | A claim marker outside a capability is `okf-claim-outside-capability`. | holds |
| 50 | okf-profile | An inline marker, or one split before `-->`, is `okf-parse-malformed-claim`. | holds |
| 51 | okf-profile | "a misspelt claim marker never reads as a plain comment" (any spacing after `<!--`) | **FALSE (G2)** for `<!--` then a newline then `raf:claim ...` |
| 52 | cli | Each option is accepted before or after the command. | holds |
| 53 | cli | The slug of a missing change record exits 2 for `docs preview`, `change impact`, `change reconcile` and `verify`. | holds |
| 54 | cli | `-q` keeps only findings. | holds |
| 55 | SECURITY.md / D12 | Only `TrustedArgv::from_command_line` builds the trusted argv outside tests; `new` is `pub(crate)` and called only in tests. | holds (code) |

## Delivered sentences reworded (validation 11A-11E) compared with `git diff origin/main -- docs/capabilities/*.md`

The seven capabilities that were delivered on main lose 84 lines in the diff. I read every removed word (`--word-diff`).

* okf-profile (11A 1-5; 11D1 fix 1): each is a widening, a type set superseded (3 to 6), or a narrowing to
  what the parser does (no error for markers in fences or code spans). None weakens behaviour.
* source-index (11A 1-3; 11D1 fixes 2-4): tense fixes, a clearer referent, and path containment stated per
  platform, which was false as delivered for macOS and for directories. The two `cfg(windows)` test
  bindings were removed and replaced by Linux siblings, with a Constraints bullet. Issue 17's scope allows
  this. It is disclosed. Accepted.
* workplan-document-model (11A 1-6; 11D1 fixes 5-7): widenings. Fix 7 discloses a pre-existing gate
  fallback ("every file stands for a tracked path outside a work tree") that this branch did not add. Accepted.
* workplan-generated-docs (11D1 fix 8): a correction. Accepted.
* workplan-reconcile (11B 1-4; 11D2 fixes 1-10, C1-2, V1-2): corrections to pre-existing code. For example
  the "listed capability that does not load" exception is `load::note_unbound`, from commit 80c303e on main.
  The stub-on-PATH Verification was reworded to how the test injects the stub, and no assertion was lost.
  Accepted.
* workplan-cli (11B 1-6; 11D2 fixes): "runs cargo with `--frozen`" is narrowed to "every cargo command the
  tool composes". This is a real narrowing that the new trusted argv introduces. It is owner-approved
  (D4/D12) and SECURITY.md states it. Accepted.
* workplan-complete-gate (11B 1-6; 11D2 fixes): accepted, except one rewording that is in no list.
  **G5:** in 21499bc (Task 10) the delivered Bounds sentence "the binding counts as failing with the time
  it ran" became "the binding counts as failing". The time is now claimed for timeouts only. The cause is
  pre-existing: `bound/run.rs:130` records `Duration::ZERO` for an output-bound kill. The validation's
  Task 10 note calls this edit only "the timeout sentence split out". It is not in 11A-11E, and no tracked
  record covers the unbuilt part. The PR residual risks mention "a kill for too much output records zero
  time".

Verdict: nothing silently weakens behaviour. One delivered clause was dropped without being listed.

## Integration and consumers

* CLI to library: `history` calls `Workplan::history` and `history::render`. `evidence record|claims|for`
  call `record_evidence`, `evidence_claims` and `evidence_for`. `assess` calls `assess_context`,
  `record_assessment` and `list_assessments`. `coverage` calls `coverage_report` and `coverage::render`.
  Each opens the gate through `Gate::workspace(root).open()`. `Lifecycle::process_obligations` is shared by
  `reconcile` (session.rs:351) and `complete` (complete.rs:231), as D17 requires. Condition 6 is
  `assessment_failures`, inside `evaluate`.
* The coverage record: `check` enforces `scope: [raf-workplan, cargo-workplan]` with `exclusions: []`
  (every public item and test of the two crates is bound). The test helpers write `scope: []`
  (`tests/support/mod.rs:190`).
* `check_inventory`: `INVENTORY.records` lists all 56 records. The capability-named record rule is intact.
  New negative fixtures: `inventory_rejects_a_missing_or_unlisted_record`. Advisory: every future
  `assess` or `evidence record` also needs an edit to `tests/docs_contract.rs`, or the pre-commit gate
  refuses the commit (validation 11E step 6 shows that refusal).
* Roadmap guard (`tests/roadmap.rs`): it pins 12 slugs as `Delivered`. The mandated reopening workflow
  therefore makes `cargo test` fail by design while a change is open. Validation Task 12 records exit 101 at
  bc3f15f, and b4ca6c8 is red the same way. Advisory: bisect-hostile, and a trap for every later change to
  these 12 capabilities.
* Makefile `docs-sync`: ran at HEAD and is idempotent (status clean before and after).
* Unloaded-document handling is inconsistent. `history` and `evidence for` exit 2, `status` warns, and
  `assess --list` marks the records stale with the reason. `evidence claims` silently answers "no claim"
  with exit 0 (G3). `check`'s coverage turns one parse error into dozens of `coverage-item-unbound`
  findings (G4). A guard that would fix both already exists: `Workplan::has_unloaded_documents`
  (model.rs:224), which rationale uses.

## ETHOS/SOLID Assessment

### SOLID Compliance
| Principle | Status | Notes |
|-----------|--------|-------|
| SRP | PASS | One module per capability (history, evidence, assessment, coverage, rationale). Renderers are separate. |
| OCP | PASS | Closed kind sets grow with pinned "ALL" tests. The claim marker is a `key=value` list, so keys can be added. |
| LSP | PASS | `Runner` fakes and the system runner satisfy one contract. Stub fixtures assert the argv. |
| ISP | PASS | The library exposes typed requests and results per command. |
| DIP | CONCERN (minor) | `coverage/render.rs` imports `crate::history::render::{string, table_cell, table_code}`, so generic escaping lives in the history module. JSON is hand-rolled in the library (history, coverage) and built with serde_json in the binary (evidence, assess). |

### ETHOS / AGENTS.md Compliance
| Section | Status | Notes |
|---------|--------|-------|
| Fail hard / no skipped checks | VIOLATION (2) | G2: a line-broken claim marker silently becomes text. G3: `evidence claims` skips a capability that does not load. |
| No fallback where the design says fail hard | COMPLIANT | `unwrap_or_default` on `record_slug` (assessment.rs:892, 941, 1053) cannot be reached: the subject is validated first. No new fallback in gate.rs. |
| One path | COMPLIANT | One trusted-argv constructor; one `process_obligations`. |
| No tracker numbers or process wording in committed files | COMPLIANT | `make issue-refs` passes. My grep of added lines and of all 42 commit messages found no tracker number, process wording or attribution. Cosmetic: raf-source `exports.rs` doc comments still say "a later field is not a breaking change", the wording the source-index assessor removed from the document. |
| No `#![feature]`, no new unsafe, no dependency change | COMPLIANT | `Cargo.lock` untouched. |
| `.deficiencies` | COMPLIANT | Notice and marker only. |
| Never bypass a hook | PROCESS NOTE | Validation "Task 12 stale cleanup" records a scratch-clone commit made with `-c core.hooksPath=/dev/null`, undone at once and redone with hooks. Nothing reached the branch, but it is the kind of bypass AGENTS.md section 3 forbids. |
| Workflow (PROPOSED blocks) | DEVIATION (LOW) | The F1/F2 reopening of entity-history wrote the new text as current text with no PROPOSED block, although the completion audit's fix 1 asked for one. The tool gates passed, and code and text landed in one commit. |

### Optionality to Remove
* None material. The `--copyright`/`--licence` pair is optional on `evidence record` and `assess`, so a
  record without SPDX lines can be written. `make reuse` catches it in this repository, and every committed
  record carries the lines.

## Owner decisions

| Decision | Honoured | Evidence |
|---|---|---|
| Coverage scope: "Workplan crates only" | YES | The record scopes the two workplan crates. `check` is clean, and the reports name the raf-okf and raf-source gaps. |
| Run the command | YES | `evidence record -- <argv>` runs it (probes 12-25). No shell, stdin null, 1800 s, 16 MiB. |
| Hosted CI counts for cross-check | YES | `raw/pr-checks.txt`: cross-check, test-windows, test-macos and the other nine jobs are SUCCESS on 43c7e02. |
| Record naming policy (D15: capability-named records, no policy change) | YES | Every reopening used a capability-named record. The `check_inventory` named-after rule is unchanged. |
| `--since` by author date | YES | Filtered in process by the author-date instant, and no argv carries it. The bound fixture `since_selects_by_the_author_date_and_not_by_the_committer_date` passes. |

## The 54 assessments (sample of 12)

All 54 are `accepted`, `by: agent`, carry SPDX lines, and are `current` at HEAD (`assess --list`). A
probe change on complete-gate at HEAD shows no `evidence-stale` and no `assessment-stale` after 43c7e02.

| Assessment | Honest? | Basis |
|---|---|---|
| entity-history#contract | yes | claims 1-11 |
| entity-history#usage | yes | the rationale example matches `git log -- rationale.rs` |
| entity-history#constraints | yes | `--since` reaches no process; heuristic limits stated |
| evidence-claims#contract | yes | claims 12-29 |
| evidence-claims#usage | yes | `status=ok` and `n=3` behave as stated |
| prose-assessment#contract | yes | claims 30-35. It says one `assessment-stale` per moved input, which is true and contradicts complete-gate#contract. |
| prose-assessment#examples | yes | line formats match |
| two-way-coverage#contract | yes (incomplete) | claims 38-44; silent on the G4 cascade |
| current-rationale#contract | yes | claims 45-48 |
| workplan-cli#contract | yes | claims 52-54; exit codes |
| workplan-reconcile#contract | yes | trace kinds and the per-problem count seen live |
| **workplan-complete-gate#contract** | **no** | G1: "each section ... is also an obligation" and the section count are false for a section with several moved inputs |
| **okf-profile#contract** | **no** | G2: "a misspelt claim marker never reads as a plain comment" (okf-profile.md:286) |

So 2 of the 13 sampled sections hold a false statement. Both are edge cases, and the agent assessors fixed
about 76 others. The PR body is candid that no human assessed any section.

## Evidence record honesty

`work/evidence/gate-kills-a-test-past-its-timeout.md`: `subject: binding:` names the real bound test, the
verdict `pass` comes from a real run, and the 16 inputs are the code bindings of complete-gate. It was
re-recorded through the tool in 8e5f568 after the exec and git digests moved. It is current at HEAD (probe
reconcile clean). The claim at complete-gate.md:410 is what the test asserts (T10 review). Honest.

The repository holds only this one claim and one record, and it uses `--from-binding`. The
`name=number`/limit path is shown only by fixtures and by my probes, not by a committed record (R2's
"at least one" is met).

## Validation (run / not run)

* Run by the author: per-task tests and reviews (T1-T10, all PASS); every `make` gate individually except
  fuzz and cross-check (11F3, Task 12); `docs-sync` idempotence; `verify` per capability before each
  completion; reconcile then complete for each of the 13 records; a completion audit with the real
  commands; a 12-capability stale probe.
* Not run locally:
  * `cross-check` (owner decision; CI now green).
  * `make fuzz` after Task 2 (the fuzz harness changed in 8844239; CI fuzz job green).
  * `make test-release` as a target. 11F3 ran a crate subset without raf-source, which has changed only in
    doc tests since Task 2; CI test-release is green.
  * The whole-repository `verify`.
  * `make check` as one chain.
  * 43c7e02, a tests-only commit after validation.md was last written. It also makes the tree snapshot
    skip files that vanish (`NotFound`), which slightly weakens the byte-identity proofs. Acceptable.
* Windows and macOS: green in hosted CI on 43c7e02.

## Deferrals: plan "Carried items" and task-review ADVISORY items

Done: T1 (sentence narrowed; Number exponent stated; no-code-binding refusal; crate-name mapping; examples
bound; `Gate` example runnable); T2 (loud limits stated in coverage Constraints; source-index tense line;
doctest order); T4 (merge wording; mixed path bases; partial clone stated; stub wording); T5 (clap and
`TrustedArgv` agree; option value `--` refused); T6 (fenced-code Constraints; six conditions; Order
paragraph; generated-only wording; assessor by kind in PR); T7 (generated-docs seventh record; roadmap
guard restored); T8 (four fenced PROPOSED blocks resolved; ATX-only stated); T9 (`assess --list` exit-2
sentence; SPDX on every re-record).

Not done and not recorded as accepted (all ADVISORY):

1. T9 A7 / plan carry: the `HistoryError::SymbolLog` message still says "have no history ... commit
   `{file}`" unconditionally (history.rs:419). Requested: "`git log -L` could not read the lines from
   `HEAD`", with committing as a conditional suggestion.
2. T7 A1: no test pins that a crate-root binding's digest is unchanged by an edit below the root (mutant
   D4 survives). `tests/digest.rs` covers modules and siblings only.
3. T7 A2: `super::` re-exports are not normalised in coverage `named_paths`. Only `self::` is stripped
   (coverage.rs:943).
4. T3 A4: the `Text file busy` flake of `sync::the_docs_sync_example_writes_...` has no lock like the other
   process tests.
5. T2: no fixture pins `binding_site` to the first carrying glob.
6. T8 A4: the `has_unloaded_documents` comment does not say that the stem-to-id comparison relies on
   `okf-id-file-name`.
7. T6 carry: the PR body does not list the prose-assessment id-rule item.
8. T1 A2: the multi-line marker (now G2, REQUIRED).

## Honest-claims check of the PR body

| PR text | Evidence | Status |
|---|---|---|
| "Merge only after the hosted cross-check, macOS and Windows jobs are green; none has run on this branch." and residual risk "Hosted CI has not run on this branch" | pr-checks: 12/12 SUCCESS on 43c7e02 | **STALE (REQUIRED update)** |
| "Delivered sentences reworded ... are listed in the stage validation notes (sections 11A to 11E)" | `.stage/` is git-ignored and invisible on GitHub. The plan's R8 check and the completion audit both require the lists in the PR body. The complete-gate Bounds clause (G5) is in no list. | **NOT MET (REQUIRED)** |
| "Local: every `make` gate was run (... test-release ...; cross-check excepted)" | No `make fuzz` after Task 2; 11F3 ran a release subset | OVERSTATED (ADVISORY: say so and cite CI) |
| "This branch fixes the three failing verify bindings of issue 17" | Two were fixed by unbinding Windows-only tests (allowed by #17's scope) | ADVISORY: "rebinds" is more exact |
| "about 80 fixes" | 8 + 30 + 38 = 76 | OK |
| "`check` exits 0 with no finding, `assess --list` shows 54 of 54 current" | Reproduced at HEAD | TRUE |
| "Nothing was dropped or moved" | Records at `<commit>^` | TRUE |
| Owner readings (D4, coverage scope, digests, exhaustive enums, condition 6, agent assessments) | Verified above; CHANGELOG states the exhaustive-match growth | TRUE |
| (absent) the nominal #9 dependency reading; the id-rule item | plan Task 12 | ADVISORY |

## Quality Gaps

### Test Coverage
- [ ] G1: no test where one section with several moved inputs is counted once in the refusal headline
  (the completion.rs:558 test has one obligation per section).
- [ ] G2: no negative fixture for `<!--` alone on its line followed by `raf:claim ...` (currently accepted
  as text).
- [ ] G3: no fixture for `evidence claims` while a capability fails to load.
- [ ] The crate-root digest pin (T7 A1).

### Maintainability
- Large modules: `graph.rs` 2047 lines (+845), `raf-source/src/exports.rs` 1537, `assessment.rs` 1112,
  `coverage.rs` 1061, `cli.rs` 1276. Test files `tests/history.rs` 1673 and `exported_items.rs` 1603.
- Duplicated logic: JSON is hand-rolled in raf-workplan (`history::render::string`) and built with
  serde_json in cargo-workplan. Coverage borrows escaping from the history module. `is_authored`
  restates raf-okf's empty-section rule (a drift test exists).
- Workflow cost: about 1400 bound tests, completions of up to 367 s, and any edit in a bound module stales
  its assessments and evidence. Disclosed. Nothing (no `check` rule or test) keeps the records current, so
  the "54/54 current" state decays silently until a change lists the capability.
- Type bindings hash only the item's text, while coverage counts them as covering their methods. A method
  body can change without staling evidence that rests on the type binding. Stated in the prose-assessment
  Constraints; asymmetric with module bindings.

## Priority Ranking

1. **[HIGH] G1**: the condition-6 headline in `change complete` counts obligations as sections. Seen
   live: one affected section with three moved inputs printed "3 sections without a current accepted
   assessment" (`lifecycle/complete.rs:72` maps every assessment obligation to a `Failure`;
   `completion.rs:347-351` counts them under the "sections" label). The delivered complete-gate Contract
   (lines 273-277) says each such section is one obligation and the count is of sections, which
   contradicts the prose-assessment Contract (one `assessment-stale` per moved input). Under the rubric
   this is a claim unsupported by the implementation. I rank it HIGH, not CRITICAL, because the refusal
   itself is right and only the number and one sentence are wrong.
2. **[HIGH] G2**: a claim marker written `<!--` / `raf:claim evidence=x` / `-->` on three lines is
   silently a plain comment. `check` exits 0 and `evidence for` reports "no claim", so the claim leaves the
   graph unnoticed. The delivered okf-profile Contract (okf-profile.md:286) and the architecture page
   (lines 268-270, "in any letter case and spacing after `<!--`") say a misspelt claim marker never reads
   as a plain comment, and the D2 goal says a misspelling never silently stops being a claim.
   Per-line scanning in `body/marker.rs` (`raf_comment` around line 170) misses it. T1 advisory 2 saw it.
3. **[MEDIUM] PR-1**: the PR body is stale about hosted CI (it says nothing has run; all 12 jobs are green).
4. **[MEDIUM] PR-2 / R8**: the reworded-sentence lists are not in the PR body (they point to git-ignored
   `.stage`), which the plan's R8 check and the completion audit require.
5. **[MEDIUM] G5**: the delivered complete-gate clause "with the time it ran" was removed in 21499bc and is
   in no rewording list. The unbuilt behaviour (zero time for an output-bound kill, `bound/run.rs:130`) is
   neither fixed nor tracked.
6. **[MEDIUM] G3**: `evidence claims <id>` iterates only loaded capabilities (`evidence.rs:775-811`) and
   answers "no claim of a capability names ..." with exit 0 while a capability fails to load. `evidence for`
   and `history` exit 2 in the same state. This breaks the fail-hard rule.
7. **[LOW] G4**: a capability that does not load makes `check` report every item it binds as
   `coverage-item-unbound` "bound by no capability" (a misleading cascade; the parse error is reported
   first).
8. **[LOW]** The roadmap guard is red by design whenever one of the 12 capabilities is reopened (bc3f15f and
   b4ca6c8 fail `cargo test`).
9. **[LOW]** The eight unaddressed carried advisories above. The `INVENTORY` coupling for every new record.
   JSON and escaping duplication. Large modules. "a later field" doc-comment wording. The F1/F2 reopening
   without a PROPOSED block. The scratch commit with `core.hooksPath=/dev/null` (process). Validation gaps
   (fuzz, test-release subset, 43c7e02) now covered by CI.

## Recommended Actions

1. G1: count distinct sections in the condition-6 headline (for example, de-duplicate
   `Failure::Assessment` by subject before counting), or relabel it. Add a fixture with one section and
   two moved inputs. Carry it through `change new workplan-complete-gate` -> implementing -> edit ->
   reconcile -> assess -> complete.
2. G2: make the scanner refuse a `<!--` line whose next non-blank text starts with `raf:` (a fail-hard
   finding with a passing neighbour). The other option is to narrow the okf-profile Contract and the
   architecture page so they state the limit; that is a narrowing and must be listed. Carry it through the
   okf-profile capability.
3. G3: make `Workplan::evidence_claims` return an error (exit 2) when `has_unloaded_documents()` is true, as
   `evidence for` does. Add a fixture and a passing neighbour. Optionally (G4) guard the coverage
   enforcement in the same way.
4. G5: list the clause removal in the PR body. Either record the elapsed time for `OutputTooLarge` kills
   and restore the sentence, or open a tracked change for it.
5. PR body: replace the CI statements with the green 12/12 result on 43c7e02. Paste the 11A-11E rewording
   lists, plus G5, into the PR body. Say that `make fuzz` and the full `make test-release` ran in CI only.
   Add the #9 dependency reading and the id-rule item.
6. Optional: word the roadmap guard as "delivered, or implementing with an open capability-named record".
   Address the eight carried advisories, or record them as consciously accepted in the PR.

## Open findings

| ID | Finding | Class | Severity |
|---|---|---|---|
| G1 | The condition-6 count in `change complete` counts obligations, not sections; the delivered complete-gate Contract says otherwise | REQUIRED | HIGH |
| G2 | A line-broken claim marker silently becomes a plain comment; the delivered okf-profile Contract and architecture page say it never does | REQUIRED | HIGH |
| PR-1 | The PR body says hosted CI has not run; all 12 jobs are green on 43c7e02 | REQUIRED | MEDIUM |
| PR-2 | The reworded-sentence lists are not in the PR body (plan R8, completion audit) | REQUIRED | MEDIUM |
| G5 | The complete-gate clause "with the time it ran" was removed unlisted; the output-kill time is neither built nor tracked | REQUIRED (list it); ADVISORY (fix or track) | MEDIUM |
| G3 | `evidence claims` silently skips a capability that does not load (fail-hard) | REQUIRED | MEDIUM |
| G4 | Coverage cascade of false `coverage-item-unbound` when a capability does not load | ADVISORY | LOW |
| A1 | The roadmap guard is red during the mandated reopen workflow | ADVISORY | LOW |
| A2 | Carried advisories not done: SymbolLog wording, crate-root digest pin, `super::` normalisation, `Text file busy` flake, `binding_site` pin, `has_unloaded_documents` comment, id-rule item in PR | ADVISORY | LOW |
| A3 | F1/F2 reopening written without a PROPOSED block; scratch commit attempted with `core.hooksPath=/dev/null` | ADVISORY | LOW |
| A4 | Validation: no local `make fuzz` after Task 2, a release subset only, 43c7e02 unrecorded (CI covers) | ADVISORY | LOW |
| A5 | Maintainability: large modules, split JSON rendering, `INVENTORY` coupling for each record, type-binding digest vs coverage asymmetry, no gate keeping records current | ADVISORY | LOW |
| A6 | A completed change's history is unreachable via `history change:`; `history cap:` omits its change records | ADVISORY | LOW |

VERDICT: BLOCKED
