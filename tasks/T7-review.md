# Task 7 review: two-way coverage

## Scope

The staged diff on top of bb415d3: 37 files, +3263/-93. Files: the `coverage`
module and its `render`, five finding kinds, the crate-name hint in `graph.rs`,
`work/coverage/default.md`, two new change records (`workplan-cli`,
`workplan-generated-docs`), and bindings and state edits in nine capability
documents. It also covers the INVENTORY, CHANGELOG and test-support changes.

Experiments ran in separate worktrees (`/tmp/t7rev`, `/tmp/t7mut`, `/tmp/t7base`
= HEAD), each with HEAD plus the staged diff applied and its own target
directory. The reviewed worktree was not modified (no git add, reset, stash or
commit). The scratch worktrees were removed afterwards.

Read: plan.md (owner coverage-scope decision, D9, D15, D16, D17, D19, Task 7,
R4, edit list, carried items), T2-review.md (the loud differences of
`exported_items`), T3-review.md, AGENTS.md / CLAUDE.md,
`workplan-two-way-coverage.md`, `coverage.rs`, `coverage/render.rs`,
`tests/coverage.rs`, and every changed test file diffed against HEAD.

## Evidence

### (1) Semantics against the document and the plan

* Surface: `exported_items` of every Lib and Bin target. Units are keyed by the
  defining `ItemId`, so an item exported under several paths is one item,
  reported under the first path. `doc(hidden)` items are left out (default
  `ExportOptions`). This matches the Contract and D9.
* Covered: `Bound::covers` checks the item and its `same_source` twins
  against the `code` binding ids and the ancestors of the **defining** path.
  A binding is resolved to its definition, so a re-export binding covers the
  item under every exported path. A crate-root binding adds nothing to
  `below`. Only `BindingKind::Code` covers items. `Test` covers by exact id
  only. `Example` covers nothing. A binding that does not resolve covers
  nothing, and `binding-missing` still fires. Two capabilities binding one
  item is fine.
* Tests: every `TestRole::TestFn` item of the package, in any target (unit
  tests included). Doc tests are not counted.
* Exclusions: an `Item` matches an exported path or the defining path. A
  `Prefix` matches below it at a `::` boundary. `matched` is recorded for
  every exclusion that hits, even when the item is also bound. The first
  exclusion in record order gives the reason. An empty reason and a
  crate-root prefix are refused by the profile. An exclusion that matches
  nothing gives `coverage-exclusion-unmatched`.
* Scope: the graph's `ScopeEdge`s (package names). A crate name such as
  `raf_okf` gets a message naming the package it belongs to; this is the T1
  carried item, done.
* `check` lists only the scope packages plus the packages whose crate path
  starts with an exclusion's first segment. Reports list every package.
* Hostile cases:
  * Bindings in a `considered` or `designed` capability are refused by the
    profile (`BindingNotEmpty`, `raf-okf/src/profile/mod.rs:526`).
    Frontmatter is the only place a binding can come from; a PROPOSED block
    cannot carry one.
  * Glob re-exports and private-module re-exports are covered by fixtures.
  * Trait impl members and enum variants are not separate surface items: no
    `Variant` and no trait-impl method appears among the 589 listed items. A
    type binding covers them anyway.
  * An `exported_items` error is a `SurfaceError` in the report and
    `coverage-surface-unreadable` in scope, never an empty surface (fixtures
    for Ambiguous and Budget).
  * The cargo-workplan binary lists 0 public items, so its code direction
    enforces nothing.
  * Macro-generated items: see REQUIRED 3.

### (2) Repository audit

* `cargo run -q -p cargo-workplan -- check`: exit 0, no output. The new binary
  run on the HEAD tree (no record) prints exactly one
  `coverage-record-missing`.
* I wrote an audit dump (a scratch test, not committed) that lists every
  exported item and every test of raf-workplan and cargo-workplan with the
  bindings that cover it:
  * raf-workplan: 589 items, 589 covered. 839 tests: 629 bound, 210
    excluded.
  * cargo-workplan: 0 items. 140 tests: 130 bound, 10 excluded.
  * Outside the scope the report names the gaps: raf-okf has 177 item gaps
    and 140 test gaps; raf-source has 35 and 215; the root package has 85
    test gaps. 0 surface errors.
* How items are covered (589 raf-workplan items):
  * 121 (20.5 %) are bound directly.
  * 195 (33 %) are covered only through a bound type.
  * 273 (46 %) are covered only through a bound module: exec 85
    (complete-gate), graph 82 (document-model; `graph` is a private module),
    coverage 41, lifecycle 41, bound 40, history 27, git 25, obligation 19,
    assessment 18, evidence 17, record 9, digest 3.
* Samples (seeded shuffle):
  * 45 ancestor-covered items: each is plausibly explained by its
    capability. The Contract states the "binding a module covers every item
    below it" rule, and complete-gate does document the invocation layer.
    The one mis-attribution is `exec::TrustedWord` / `TrustedArgvError`
    (evidence-claims material), covered by complete-gate through
    `raf_workplan::exec`.
  * 45 bound tests: each is bound to the capability whose Verification it
    belongs to.
* Binding a module is a legitimate reading of "binds" for **coverage**: the
  Contract says so, and the coverage is not vacuous. A file-module binding is
  **vacuous as a staleness input**, though: see REQUIRED 4.
* The 25 `<module>::tests::*` exclusions take out 219 entries. All are unit
  tests in `src/`. No public item and no integration test is hidden. The single
  reason is untrue for several modules: see REQUIRED 1 and 2.

### (3) The departures

* **(a)** The four `considered` capabilities set to `implementing`: this is
  forced by the profile rule above. Each has its open record
  (`work/changes/workplan-{entity-history,evidence-claims,prose-assessment,two-way-coverage}.md`).
  `check` is clean with their PROPOSED blocks in `implementing`. The
  only text outside a PROPOSED block that changed is frontmatter. Honest.
  Plan impact: Task 11 step (1) is done for four of the five;
  `current-rationale` must be flipped by Task 8 when it binds.
* **(b)** `workplan-cli` and `workplan-generated-docs` reopened, following the
  D15 rule (a change to a delivered capability puts it under a capability-named
  record):
  * `contract_effect: none`, `affects: #verification`. The only document
    change is bindings and the generated matrix and relations rows. This
    is honest in shape.
  * Both records already mark their items `outcome: implemented`, while every
    other open record leaves outcomes for Task 11.
  * The `workplan-cli` record has an untrue claim: see REQUIRED 2.
  * Plan impact: Task 9 must widen the existing `workplan-cli` record (a
    `change new` with that slug would refuse). `workplan-generated-docs` is
    a seventh D16 record that Task 11 must complete. Condition 6 will want
    assessments of both Verification sections.
  * Time: `verify workplan-generated-docs` took 78.5 s for 83 bindings
    (about 0.95 s per binding, warm cache). The workplan capabilities hold
    about 762 test bindings, so one pass over all of them is about 13
    minutes. Each `complete` of a record that lists several capabilities
    runs all of their bindings (cli alone 129, document-model 240). Task 11
    should budget about 1 to 2 hours of wall time for the 12 completions.
* **(c)** `tests/roadmap.rs` now asserts `delivered + implementing != 0`. At
  HEAD exactly two capabilities were delivered, and both are reopened here, so
  the old assertion could not hold. This is acceptable as a transitional
  invariant (see ADVISORY 4).

### (4) Findings kinds

* The five kinds are in `KINDS` and `OWN_KINDS` (39 -> 44). Each has a case in
  `coverage_cases()` with file and location, and `kinds.rs` checks both
  directions. Each has a fix text.
* `coverage-record-missing` fires only when a `cap:` file is present and no
  record file is present. A broken record gives its own finding, not
  "missing"; tested. A repository without capability documents gets no
  finding; tested (mutant M5 killed).

### (5) Fixture helper and changed tests

* One helper per crate writes `scope: []` and `exclusions: []`:
  `support::open_coverage` and `with_open_coverage` in raf-workplan, and
  `OPEN_COVERAGE` in cargo-workplan. It is used by `fixtures::valid`,
  `synced::unsynced`, `changes::repo`, `Project`, and the `complete` and
  `assessments` fixtures. No fixture relies on a default.
* Diffed against HEAD:
  * `gate.rs` and `lifecycle.rs`: `work` becomes `work/changes`, because
    the record lives under `work/`.
  * `records.rs` and `cases.rs`: `scope: [demo]` becomes `[]` with a
    matching exclusion. The scope-edge test re-adds `[demo]`.
  * `a_repository_without_record_directories...` now expects the one
    `coverage-record-missing`.
  * `graph.rs`: an exclusion was added for `demo::inner`.
  * None of these weakens an assertion except the roadmap guard (3c).

### (6) Determinism, render, scale, time

* The two-run equality test passes for the report, the table and the JSON
  (mutant M7 killed).
* Render reuses the history escaping. A reason holding `|`, `<!--` and `[x]`
  is escaped in the table and kept raw in the JSON; tested.
* Scale fixture: 3060 items, 1200 tests, 600 exclusions, under a 60 s bound
  (loose; it runs in about a second). The exclusion match is
  O(entries × exclusions).
* On this repository, `coverage_report()` takes 5.9 ms (release). Total
  `workplan check` (release) is 1.10-1.22 s staged against 0.76-0.92 s at
  HEAD, about +0.3 s. That includes the 460 new bindings and the larger
  generated blocks. The debug times were noise-dominated (5.9-6.3 s staged,
  7.6-8.6 s at HEAD).

### (7) Documents

* Outside PROPOSED blocks, the only changes are `state`, `bindings`, the
  generated matrix and relations rows, and the index rows (the "Delivered"
  group emptied; "bindings.examples is empty" gaps). All two-way-coverage
  Contract, Usage, Examples, Constraints and Verification edits are inside
  PROPOSED blocks. The edit-list row "check enforces the record's scope ..."
  is carried out.
* INVENTORY: 11 implementing, 0 delivered, 1 considered, `records:
  ["coverage:default"]`.
* CHANGELOG: one entry; it is accurate ("every public item and integration
  test" is bound).

### (8) Mutants

The tests `coverage`, `kinds` and `records` were run against each mutant.

| Mutant | Result |
| --- | --- |
| M1 descendant coverage dropped | killed (7 tests) |
| M2 prefix match without the `::` boundary | **survived** |
| M3 enforcement widened to every package | killed |
| M4 cfg(test) unit tests dropped | killed (8) |
| M5 record-missing without the capability-file condition | killed |
| M6 `below` keyed on the written binding path, not the definition | killed (5) |
| M7 report order reversed | killed (6) |
| M8 (extra) an `examples` binding covers items | **survived** |
| M9 (extra) the last matching exclusion gives the reason | **survived** |

### (9) Hygiene

* All new files carry SPDX lines.
* No tracker number, process wording or attribution in the added lines.
* No dependency change and no `unsafe`.

### (10) Gates (in `/tmp/t7rev`, CARGO_BUILD_JOBS=6)

All passed (rc 0):

* `make fmt`, `make clippy`, `make doc` (`-Dwarnings`).
* `make test`: 1852 passed, 0 failed, docs_contract and repo_gates included.
* `cargo test -p raf-workplan`: 871 passed in debug and 871 in release.
* `make test-release`, `deny`, `reuse` (429/429), `msrv` (1.98.1),
  `issue-refs`.
* `make docs-sync` run twice: no working-tree change either time.
* `cargo run -p cargo-workplan -- check`: clean.

## Findings

### REQUIRED

1. **The single exclusion reason is untrue for unit tests that are the only
   proof of documented guarantees.** The reason claims "the behaviour the
   capabilities document is bound through the integration tests". That is
   false here:
   * `cap:workplan-evidence-claims` binds 6 tests. Its Verification bullet
     "The run of the command: its argv exactly, standard input closed,
     bounded output, a kill at the time limit (recorded as fail), the
     workspace root ..., each fault of the printed values is an error beside
     a passing neighbour, and a command that cannot start writes no record"
     is proven only by tests that are excluded:
     * `raf_workplan::evidence::tests::*`, for example
       `the_argv_reaches_the_process_exactly_with_no_shell_of_the_workplan`,
       `the_command_runs_in_the_root_with_its_input_closed_and_the_callers_environment`,
       `a_run_killed_at_its_time_limit_records_fail_and_its_neighbour_passes`,
       `each_fault_of_the_printed_values_is_an_error_beside_a_passing_neighbour`
       and `a_command_that_cannot_start_writes_no_record`;
     * `evidence::measure::tests::*`.

     `tests/evidence.rs` has no test of the run.
   * prose-assessment's bullet "adding code spans that hold only last
     segments never removes an input" is proven only by
     `assessment::tests::widening_a_section_with_weak_names_never_removes_an_input`,
     which is excluded.

   Unit tests can be bound: `cli::tests::a_slug_that_starts_with_a_dash...`
   and `exit::tests::the_three_exits_have_the_documented_codes` are bound,
   while their siblings are excluded.

   Fix: bind each unit test that supports a documented guarantee in the
   `bindings.tests` of that capability. At least bind all of
   `evidence::tests`, `evidence::measure::tests` and the assessment widening
   test. Then go through the other 23 prefixes: each remaining exclusion
   needs a reason that is true for every test it takes out, which means
   per-module reasons where they differ.
2. **`change:workplan-cli` item 2 is untrue while it is marked
   `implemented`.** It says every test of the root discovery in raf-workplan
   is bound. Only the 8 integration tests `test_crate::root::*` are. The 19
   unit tests `root::tests::*` (5) and `root::json::tests::*` (14) are
   excluded. Bind them, or reword the claim to "every integration test".
   Also leave `outcome` unset until the record is reconciled, as every other
   open record does, or say why these two differ.
3. **Coverage silently drops part of an in-scope crate's public surface.**
   The `validated!` macro in `exec/pattern.rs` generates 11 public types,
   which `raf_workplan::exec` re-exports: `Commit`, `Date`, `DocFilter`,
   `ExampleName`, `Filter`, `Identifier`, `Package`, `RefName`, `SourcePath`,
   `TargetName` and `TreePath`, each with `new`, `as_str` and `Display`.
   * `exported_items` returns them as `Reexport { item: None, written:
     "pattern::X" }`. `coverage.rs` ignores `Exports::reexports` entirely.
   * So they are neither covered, nor a gap, nor excluded, nor an error.
     The Contract's "the coverage reports name every gap ... complete or
     states what is missing" is false for them, and the Constraints do not
     state the limit.
   * Fix: report each public re-export of the target whose path the index
     cannot follow and that is not a crate outside the workspace (as `syn`
     in raf-source is). Make it a report entry, plus a finding for a crate
     in `scope`, with a fixture (macro-generated item re-exported) beside a
     passing neighbour. Then bind or exclude the 11 types. If the owner
     prefers to state a limit instead, it must be stated in the Constraints
     and listed in the report. A silent skip is not acceptable.
4. **"Binding a module" means two different things.** Coverage reads a module
   binding as covering every item below it: 273 of the 589 raf-workplan items
   are covered only that way. But `Digest::symbol` of a file-module binding
   hashes only its declaration (`raf_workplan::exec` = 13 bytes,
   `pub mod exec;` in lib.rs; the same for graph and lifecycle).
   * Evidence inputs (the digests of the code bindings of each `--for`
     capability) therefore do not go stale when the code of a bound module
     changes. Neither do assessment inputs (all code-bound symbols of the
     capability when a section names none). This affects the capabilities
     bound mainly by module in this task: history, evidence, assessment,
     coverage, exec, bound, lifecycle, obligation, git, graph, record,
     digest, refs, plain and root.
   * That defeats the R2/R3 property "changed bound code => stale, naming
     the symbol" for Task 10's real records.
   * Fix: make the symbol text of a module binding the module's content (an
     out-of-line module's file and the files below it, or the items below
     it), with a digest, evidence and assessment fixture showing that an
     edit inside a bound file module moves the input. Alternatively, replace
     the module bindings with type and function bindings. Either way, one
     meaning of "binds a module".

### ADVISORY

1. Three mutants survived, so three tests are missing:
   * M2: a `<module>::*` prefix must not match a sibling sharing the text
     (`demo::body::*` against `demo::body_x::f`).
   * M8: an item bound only in `examples` is a gap.
   * M9: with two matching exclusions, the first one's reason is shown.
2. Update the plan:
   * Task 9 widens the existing `workplan-cli` record.
   * Task 11 completes seven D16 records (`workplan-generated-docs` added).
   * Task 11 step (1) is already done for four capabilities.
   * Task 8 flips `current-rationale` to `implementing` when it binds.
   * Budget the `complete` runs (about 0.95 s per bound test or example).
3. The `this_repository_...` test does not check R4's "reports name raf-okf
   and raf-source gaps". It checks only that those packages are out of scope.
   Assert that their gaps are non-empty.
4. Restore the roadmap guard to `group(Delivered)` non-empty in Task 11, once
   the states return to `delivered`.
5. A binary target's public surface is empty in practice (cargo-workplan: 0
   items), so the code direction enforces nothing for a binary. Say so in the
   Contract's public-surface paragraph.
6. `exec::TrustedWord` and `exec::TrustedArgvError` are explained by
   evidence-claims but covered by complete-gate through `raf_workplan::exec`.
   Bind them with `exec::TrustedArgv` in evidence-claims.
7. The evidence-claims staleness and removal bullets are proven by tests bound
   to reconcile (`tests/evidence.rs`). Coverage is satisfied, but the
   generated verification matrix of evidence-claims will not list them.
   Consider binding them in both capabilities.
8. Redundant bindings (`exec::Invocation` beside `exec`, `lifecycle::Lifecycle`
   beside `lifecycle`) are harmless, but they double the module cost in
   `verify` and the digest inputs.

Round-one verdict: BLOCKED (REQUIRED 1 to 4).

## Round two: recheck of the fixes

The staged diff now has 41 files, +4196/-109. It was re-applied to scratch
worktrees (HEAD + `git diff --cached`) with their own target directories.

* The audit dump on the new tree:
  * raf-workplan: 601 items, all covered. 845 tests, all bound.
  * cargo-workplan: 0 items. 140 tests, all bound.
  * The record now has `exclusions: []`.
  * Outside the scope: 212 code gaps and 440 test gaps.
  * My own scratch audit test was reported at once as
    `coverage-test-unbound`: enforcement is live.
* Unit-test bindings per module, mapped to the capability that documents
  each module:
  * cli, exit, render, root, root::json -> cli;
  * bound, exec, lifecycle::completion, staged -> complete-gate;
  * git, lifecycle, lifecycle::changed, obligation, obligation::trace ->
    reconcile;
  * git::log, history::render -> entity-history;
  * evidence, evidence::measure, exec::trusted -> evidence-claims;
  * assessment -> prose-assessment;
  * generated -> generated-docs;
  * bindings, digest, finding, graph -> document-model.
* Module-digest edge cases, each checked against the staged code with a
  scratch test (MOVED = the digest of the bound module changes):
  * edit of a `#[path]` child file: MOVED;
  * edit of the file of a `#[path]` module: MOVED;
  * inline module, edit of its file child: MOVED;
  * inline module, edit of a crate-root sibling: SAME;
  * edit of a cfg-gated child: MOVED;
  * edit of a cfg(test) child file: MOVED;
  * edit of a `mod.rs` grandchild: MOVED;
  * file shared by the lib and a bin through `#[path]`: the lib module MOVED,
    and the bin module has its own digest;
  * child file contents swapped: MOVED;
  * CRLF-only change: SAME;
  * crate root binding, child edit: SAME (as stated in the doc);
  * module re-exported as an alias, edit: MOVED.

* Unit-test binding honesty. I sampled 40 bound unit tests (seeded shuffle)
  and also read the full lists for git, lifecycle, staged, digest, graph and
  finding. Each test proves behaviour that the bound capability's Contract or
  Verification describes:
  * cli parsing, exit codes and render -> cli;
  * the list and run proof of bound runs, doc-test refusal and benchmark
    refusal -> complete-gate;
  * the evidence run, faults and argv -> evidence-claims (this closes
    round-one R1);
  * weak and strong names and authored text -> prose-assessment;
  * the merge base, the version and the default branch -> reconcile;
  * log parsing -> entity-history;
  * SHA-256 vectors and the encoding -> document-model.

  The mapping follows the module each test exercises, and each module was
  already bound in code to the same capability; it is not a pile on one
  capability to silence the check. One debatable placement: the two tracked-path
  tests in `git::tests` go to reconcile, while `git::tracked_paths` is bound
  to document-model. Reconcile's Contract does use the tracked paths, so this
  is acceptable (ADVISORY 3).
* `verify` actually runs the new unit-test bindings:
  * `verify workplan-cli`: 162 pass, 0 other, 282 s under load average
    15-26. This includes the bin unit tests (`cli::tests`) and the lib unit
    tests (`root::tests`, `root::json::tests`).
  * `verify workplan-evidence-claims`: 32 pass, 31 s.
* R2: both records have their outcomes removed, and item 2 of `workplan-cli`
  now says "unit tests included". That is true: all 19 root unit tests are
  bound. The generated-docs text says "unit tests included" too, and its 4
  `generated::tests` are bound.
* R3: a new `GapKind::Reexport` ("unresolved re-export") and the finding
  `coverage-reexport-unbound` (45 kinds, with a fixture case and both
  directions):
  * A re-export with `item: None` whose path is local (`crate`, `self`,
    `super`, a workspace crate name, or a name the declaring module
    resolves) is reported.
  * It is covered by a module binding that its exported or named path is
    below, or by an exclusion.
  * `serde::Serialize`, which is external, is not counted.
  * The fixture `made` covers the gap, the finding, the module that covers it
    and both exclusion forms. On this repository, 12 entries are listed (601
    items against 589 before), all covered by `raf_workplan::exec`.
  * Limit: a `super::` written path is not normalised in `named_paths`
    (ADVISORY 2).
* R4: `Digest::symbol` of a `Mod` item below a crate root adds the whole text
  of every file reached under that module path, or below it, in the same
  target. Each such file is one field, keyed by (file, 0, whole-file), sorted
  and de-duplicated, so the result is deterministic. My edge cases are listed
  above; every one behaved as stated.
  * Symlinks are handled by the source index: an in-root symlink is
    accepted and read through its link; a link pointing outside the root is
    refused (existing source-index tests).
  * The documents and the code agree: the PROPOSED text of document-model
    and prose-assessment and the module doc of `digest` say the same.
  * Fixtures: digest (own file, child, sibling, determinism), evidence
    (`an_edit_inside_a_bound_module_makes_its_record_stale...`) and
    assessment (`an_edit_inside_a_bound_module_moves_the_assessment...`).
* Mutants, run against the tests `coverage`, `kinds`, `records`, `digest`,
  `assessments` and `evidence`:
  * Killed: M1 descendant coverage dropped; M2 prefix without the `::`
    boundary (now killed); M3 scope widened; M4 unit tests dropped; M5
    record-missing condition; M6 re-export mapping; M7 order; M8 examples
    covering items (now killed); M9 last exclusion's reason (now killed);
    R1 unresolved re-exports skipped; R2 named path ignored; D1 module files
    not hashed; D2 child modules not hashed; D3 every file of the target
    hashed.
  * Survived: **D4**, where a crate-root binding also hashes the files
    below it (ADVISORY 1).
* Gates, all rc 0:
  * `make fmt`, `make clippy`, `make doc`.
  * `make test`: 1859 passed, 0 failed, docs_contract and repo_gates
    included.
  * raf-workplan: 878 passed in debug and 878 in release.
  * `make test-release`, `deny`, `reuse`, `msrv` and `issue-refs`.
  * `make docs-sync` run twice: no working-tree change.
  * `cargo run -p cargo-workplan -- check`: clean.
* `check` time on this repository, release, interleaved with HEAD under load
  average 15-23: HEAD 0.95-1.07 s, staged 0.85-1.29 s, so the overhead is in
  the noise. The debug binary took 9.5-10.6 s under the same load.
  `coverage_report()` takes 6.1 ms.
* Documents: outside PROPOSED blocks, still only `state`, `bindings`, the
  generated rows and the index rows changed.

### Round-two findings

#### REQUIRED

None. Round-one REQUIRED 1 to 4 are fixed.

#### ADVISORY

1. Mutant D4 survives. Add an assertion that a crate-root binding's digest
   stays the same when a file below the root is edited (`Digest::symbol(src,
   "demo")` is unchanged after an edit of `src/tools.rs`). The behaviour is
   documented, but nothing pins it.
2. `named_paths` does not normalise a written path that begins with
   `super::`. It builds `<module>::super::X`, so only the exported path's
   module can cover such a re-export. Read `super::` as the parent of the
   reaching module path, with a fixture.
3. `git::tests::a_tracked_path_must_be_utf8_and_a_relative_path_of_the_tree`
   and `the_tracked_paths_are_the_nul_separated_listing_of_the_index` could be
   bound to document-model, which binds `git::tracked_paths`, as well.
4. Lifecycle cost: the workplan, okf-profile and source-index capabilities
   now hold 1383 bound tests and examples. `verify workplan-cli` took 282 s
   for 162 bindings under load. Task 11's completions (12 records, several
   listing many capabilities) should budget 2 to 3 hours of wall time, or
   group the records.
5. Still open from round one:
   * Plan updates: Task 9 widens the existing `workplan-cli` record; a
     seventh D16 record (`workplan-generated-docs`); Task 11 step (1)
     already done for four capabilities; Task 8 flips `current-rationale`.
   * Restore the roadmap guard to "something is delivered" in Task 11.
6. A module binding now makes evidence and assessment inputs stale on any edit
   in the module's files, its `#[cfg(test)]` tests included. This is
   conservative and correct, but expect re-records and re-assessments in
   Tasks 10 and 11 after every edit of a bound module.

VERDICT: PASS
