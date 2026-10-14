# T7 review: the Gate, and docs_contract on raf-workplan

Reviewer: independent. Subject: the staged change on top of df2c46b (8 files). Method: the staged diff was applied to a
copy with its own git dir (`/tmp/t7rev`, one throw-away commit made there so the real `.githooks/pre-commit` could run)
and the old gate was run in a second copy at df2c46b (`/tmp/t7old`). Nothing in the real worktree was touched.

## Scope

1. Disposition table against the old file, test by test.
2. Rule parity by mutation of the repository's real documents, new gate against old gate.
3. `Gate` (G7, G8): false-green protection, tracked-path sources, symlinks, the Usage example, a member crate directory.
4. The hook path (snapshot procedure, and the real hook).
5. `docs/capabilities/index.md` (G12, G23) and the `docs_contract` module docs.
6. Gates on the staged state.

## Evidence

### (1) Dispositions

- Old `#[test]` list rebuilt from `git show HEAD:tests/docs_contract.rs`: 141 functions (the raw `#[test]` grep gives 145
  lines; four are text inside fixtures). The table has 141 rows with the identical names in the identical order. No
  duplicates, none missing.
- Counts verified mechanically: 74 moved, 49 replaced, 18 kept = 141.
- Moved: all 74 targets exist by name, each preceded by `#[test]`. 70 keep the old name; 4 are renamed (see A1).
- Kept: all 18 are present in the new `tests/docs_contract.rs`. The new file has 26 `#[test]`s = 18 kept + 8 new, as claimed.
  Bodies were diffed against the old ones: 11 are byte-identical; 7 differ only as described below.
- Replaced: the 54 fixture names cited exist among the 133 fixture cases (enumerated with a temporary test in the copy, then
  removed) and the kind each fixture produces equals the rule the row names, for every pair.
- 25 old bodies compared against the new ones in detail (10 moved at random, 5 more moved, 3 kept, and 7 replaced rows
  including the broad ones: 70, 96, 97). Moved ports are faithful: same inputs, same expectations; message-text
  assertions became kind-and-location assertions, which are stricter. Deviations that are not pure ports are A2 and A3.
- Kept tests that changed in meaning: `rejects_a_tracked_symlink_that_nothing_links_to` and
  `rejects_non_markdown_file_in_a_document_directory` now call the helper directly instead of the whole checker
  (see R2); `inventory_accepts_a_matching_tree` no longer also runs the model over the fixture (recorded in the
  table as deliberate difference 4); `repository_documents_satisfy_the_profile` and
  `repository_bindings_resolve_in_the_source_index` are stricter than before (roots `["."]`, package-set assertion,
  per-capability binding counts, every inventory capability loads).
- Balance: new + moved fixtures/tests are well above the 123 removed (247 integration tests, 133 fixture cases).

### (2) Rule parity by mutation (real documents, copies; new gate vs old gate)

Each mutation was applied to the same real file in both copies (staged in the copy's index), then `docs_contract` and
`repo_gates` were run. Direct mode, and for 13 of them also the hook mode (`checkout-index` snapshot + `RAF_GATE_ROOT`).

Both gates failed with a relevant finding (41 mutations; 46 effective mutations were run in all, one more was a no-op and is not counted):
broken anchor; link to an untracked file; delivered capability with a PROPOSED block; depends cycle (delivered-involved
and designed-only); binding to test-only code; unclosed generated block; issue-number token; id differing from file name;
tracked symlink; non-Markdown file in a document folder; uppercase `.MD` in a document folder; typed page outside the
folders; process wording in README; split link; placeholder token; unlisted document (inventory); state change
(inventory plus binding rules); link leaving the repository; broken link in `crates/**.md`; missing section; empty
section; delivered capability listed by an open change record; unknown frontmatter key; dangling depends; unterminated
quote in frontmatter; unresolved binding; nested document; stray Markdown without frontmatter; broken `href`;
broken reference definition; empty link target; bad `contract_effect`; bad `affects` section; unterminated quote in
an `items` claim; empty flow entry; PROPOSED block in README; malformed and unclosed frontmatter in `docs/`.
New-only rules behaved as designed: hand-written index text, missing index, stale catalogue.

Old gate failed, NEW GATE DID NOT (parity gap, R1):
- `docs/notes.md` with well-formed frontmatter `title: x` (no `type`). Old: ``missing required key `type` ``. New: all green.
- `NOTES.md` with `type: Bug`. Old: ``bad type `Bug` ``. New: all green.
- `work/notes.md` with `id: x`. Old: ``missing required key `type` ``. New: all green.
- `README.md` given a frontmatter block `foo: bar`. Old: ``missing required key `type` ``. New: all green.

Cause: the old gate validated every Markdown file at the top level, under `docs/` and under `work/` that has
frontmatter as a typed document, so none could be an untyped or unknown-typed page. The new `outside_violations`
only reports a page whose `type` is already a known document type, and `plain::check` treats frontmatter outside the
folders as plain Markdown (pinned by `reads_markdown_outside_the_document_directories_with_or_without_frontmatter`).
The dispositions note and the deviation list describe only the known-type half.

Wiring mutations of the new `docs_contract.rs` itself (each run, then the file restored):
- remove the `plain::check` loop: caught (2 tests fail); drop `.with_symlinks`: caught; roots `["crates"]`: caught.
- remove `symlink_violations`, `layout_violations`, `outside_violations` or `check_inventory` from
  `repository_documents_satisfy_the_profile`: ALL 26 TESTS STAY GREEN (R2). In the old file the first three lived in
  `check`, which the repository test and the fixtures shared, so removing them failed fixtures.

### (3) Gate

- G8: a missing `docs/capabilities`, an empty one, a missing directory, an unloadable source workspace are `GateError`s
  (10 tests in `tests/gate.rs`; also run by hand). A member crate directory of a workspace (git and non-git) panics
  with ``/…/crates/m/docs/capabilities does not exist … pass the repository root``: it does not pass vacuously.
- Usage example copied verbatim into `tests/workplan.rs` of a scratch single-package repo (dev-dependency on the
  crate): compiles, reports findings until the index is synced (`docs_sync` example) and a `tests` binding exists, then
  passes. Inside a `git init` repo: an untracked link target is `link-broken` until `git add`, then clean; a link to a
  symlink is `link-symlink`; with `.git` removed the on-disk symlink is still `link-symlink` and a link to a missing
  file is `link-broken`. The fallback (no `.git`: every file stands for a tracked path) is documented in the type
  docs, the crate docs and the changelog.
- Gate runs `Workplan::check` over the document folders only, so a broken link in `README.md` is `docs_contract`'s
  finding, not the Gate's (confirmed; by design).
- Silent degradation: with git present but failing, the Gate takes the fallback without saying so (A4).
- `.worktrees/` copies and a `target/` copy inside the repository did not change the loaded package set.

### (4) Hook path

- Snapshot procedure of `.githooks/pre-commit` run through my harness for 13 mutations: all failed `docs_contract`
  (or `repo_gates`) with a relevant finding; the unmutated snapshot passes (26 + 57 tests).
- The real hook: `git commit` in the copy with the clean staged state passed fmt, reuse, gates. A commit with a broken
  anchor was refused (`anchor-broken … okf-profile.md:103`, "pre-commit: gates failed"). A partially staged commit (good
  index, broken working tree) passed, so the hook judges the snapshot.

### (5) index.md

- `rm docs/capabilities/index.md; make docs-sync` recreates it byte-identical to the staged file (sha256 1a226bfb…);
  a second run changes nothing. 21 capability rows (= the 21 INVENTORY capabilities), groups Delivered, Designed,
  Considered (no Implementing group because none is implementing).
- SPDX header: two single-line comments (copyright, `CC-BY-4.0`); `make reuse`: 315/315 files compliant.
- INVENTORY row: `generated: ["docs/capabilities/index.md"]`, exact in both directions (two new tests, shape test).
- No issue numbers or process wording (`make issue-refs` passes; manual grep of the staged lines found none).
- Module docs of `docs_contract` are accurate for the file, except for the one omission in A5.

### (6) Gates on the staged state (copy)

| Gate | Result |
|---|---|
| `make fmt` | pass |
| `make clippy` (workspace, all targets, `-D warnings`) | pass |
| `cargo test --locked --workspace` | 997 passed, 0 failed (docs_contract 26, repo_gates 57, raf-workplan gate 10, Gate doctest compiled) |
| `cargo test -p raf-workplan --release` | 257 passed, 0 failed; `-p raf-source --release` pass |
| `RUSTDOCFLAGS=-Dwarnings cargo doc --workspace --no-deps` | pass |
| `make reuse` | pass |
| `make msrv` | pass |
| `make issue-refs` | pass |
| `docs_contract` + `repo_gates` in a checkout-index snapshot | pass |

## Findings

### REQUIRED

**R1. Parity gap: frontmatter on a page outside the document folders is no longer refused.**
The old gate failed every top-level, `docs/` and `work/` Markdown file with frontmatter that was not a valid typed
document (no `type`, unknown `type`, missing keys). The new gate only refuses a page that declares a known type
(`outside_violations`). Reproduced with four real-document mutations (list above): old red, new green.
Fix in `tests/docs_contract.rs` (repository policy, where the typed-page rule already lives): in `outside_violations`,
refuse any page in `is_documentation` and outside the document folders for which `Document::parse` succeeds (frontmatter
present), whatever its `type`; keep the message for a known type. Add the negative cases (no `type`, `type: Bug`, a
frontmatter block in README) to `rejects_a_typed_document_outside_the_document_directories` (rename it if it now covers
any frontmatter), and update item 1 of the "Deliberate differences" list in the dispositions and the module docs.
Crate fixtures under `crates/**` stay exempt as before.

**R2. The wiring of the repository-policy checks is unprotected.**
`repository_documents_satisfy_the_profile` calls `symlink_violations`, `layout_violations`, `outside_violations` and
`check_inventory`; deleting any of the four calls leaves all 26 tests green. The kept tests
`rejects_a_tracked_symlink_that_nothing_links_to` and `rejects_non_markdown_file_in_a_document_directory` used to go
through the whole checker and so protected this; now they call the helper alone. (For `check_inventory` the old file had
the same weakness, but the other three are a loss of protection caused by this change.)
Fix: extract one function `violations(&Checkout, &Workspace) -> Vec<String>` (model findings, symlinks, layout, outside,
inventory) used by the repository test, and have the two kept tests and the typed-outside test drive it with small
`Checkout` fixtures (the `checkouts` module already builds them). Re-run the four deletion mutations above; each must
turn a test red.

### ADVISORY

- **A1.** Four moved tests changed name (`accepts_moved_outcome`, `accepts_a_generated_capability_index`,
  `accepts_a_generated_index_with_an_spdx_header`, `a_binding_to_a_macro_generated_function_is_not_found`), against the
  plan's "each ported test keeps its name". All four are listed in the table with their new names; no action needed
  beyond awareness.
- **A2.** Two ports are narrower than their originals. `accepts_a_moved_outcome_that_names_an_existing_change` moves to
  the change record itself (`add-alpha`) where the old test created a second record `other-work`; a lookup that always
  succeeded for the record's own slug would pass. `a_generated_index_may_carry_a_licence_header` drops the single-line
  `<!-- SPDX-… -->` form that the old test also accepted (the real `index.md` and the sync tests cover it). Restore both
  cases.
- **A3.** Replaced rows collapse multi-position matrices into one fixture per rule: old tests 70 (seven value
  positions), 96 (nine item shapes), 97 and 93 (list shapes) now have one fixture per rule at one position. The
  per-position behaviour is raf-okf's (`refusals.rs` `unterminated_quotes_are_refused_in_flow_list_item_and_nested_item_positions`,
  `profile.rs` `WrongShape` cases), and four real-document mutations in different positions were caught, so this is
  acceptable; the table could say that the position matrix is owned by raf-okf.
- **A4.** `Gate` falls back to "every file stands for a tracked path" whenever `git rev-parse` does not report a work
  tree, including when git is installed but fails (safe-directory refusal, bad `GIT_DIR`, no `git` on the path). Shown:
  a link to a file that is on disk and not tracked is `link-broken` normally and passes under `GIT_DIR=/nonexistent`.
  The type docs describe the fallback only as "not inside a git work tree". Either distinguish "not a repository" from
  "git failed" (the latter an error, as the plan's fail-hard rule suggests) or document the failing-git case too.
- **A5.** The `docs_contract` module docs list the policies that stay but omit the refusal of a typed page outside the
  document folders (and will need to cover R1's wider rule).
- **A6.** The Gate checks the document folders only; links in README and other plain Markdown are `docs_contract`'s.
  That is the design (G4) and is stated; noted so nobody expects the Gate alone to cover them.

## Result

The change is otherwise sound: the table is complete and exact, every moved and replaced target exists and is
equivalent, 41 of the 45 old-gate-caught real-document mutations are caught by the new gate as by the old gate (the other 4 are R1), the Gate fails
hard where it should, the hook path works (including the real hook), `index.md` is reproducible, and every gate is
green. Two defects remain that weaken the gate relative to the old one.

VERDICT: BLOCKED

## Recheck

Subject: the staged state after the fixes for R1, R2 and A1 to A4 (10 files). Method as before: a copy with its own git dir
(`/tmp/t7rev2`, one throw-away commit so the real hook ran), the old gate in a second copy at df2c46b.

### R1: frontmatter outside the document folders

The four mutations that were red-old/green-new (`docs/notes.md` without `type`, `NOTES.md` with `type: Bug`,
`work/notes.md` with `id: x`, README with `foo: bar` frontmatter) are now red in the new gate, in direct mode and in the
checkout-index snapshot, each with ``frontmatter belongs only in the documents of docs/capabilities, work/changes,
docs/architecture; this page is outside them``. Crate fixtures under `crates/**` stay exempt. The real repository is green.

### R2: wiring of the repository policies

`violations(&Checkout, &Workspace, &Inventory)` is used by the repository test and by the fixture tests (temporary
`#![allow(dead_code, unused_imports, unused_variables)]` in the copy only, file restored after each run). Deleting one call:

| Deleted | Tests that turn red |
|---|---|
| `symlink_violations` | 2 (`rejects_a_tracked_symlink_that_nothing_links_to`, `the_checker_judges_links_against_the_tracked_paths_and_symlinks`) |
| `layout_violations` | 1 (`rejects_non_markdown_file_in_a_document_directory`) |
| `outside_violations` | 1 (`rejects_a_typed_document_outside_the_document_directories`) |
| `check_inventory` | 8 (the inventory family) |
| the model findings in `judge` | 3 |
| the `plain::check` loop | 2 |

Previously the first four left all 26 tests green.

### Dispositions

- The table still has 141 rows, with names and order identical to the old `#[test]` list rebuilt from
  `git show HEAD:tests/docs_contract.rs`. 74 moved (all 74 targets exist by name with `#[test]`, every one keeps its
  original name now), 18 kept (all present in the new file, which has 26 tests), 49 replaced.
- Renamed-back and re-shaped ports read: `accepts_moved_outcome` moves to a second record `other-work` that exists and
  lists the capability (A2 closed); `accepts_a_generated_index_with_an_spdx_header` accepts the multi-line and the
  single-line header (A2 closed); `accepts_a_generated_capability_index` and
  `a_binding_to_a_macro_generated_function_is_not_found` carry their old names.
- The 54 fixtures cited still exist among 133 cases, and each produces the rule its row names.
- A3: the 37 distinct raf-okf tests the rows cite (`crates/raf-okf/tests/{profile,refusals,markers,index}.rs::...`) all exist
  by name with `#[test]`.

### Parity set (46 effective mutations of the real documents, plus one no-op)

| Gate | Caught |
|---|---|
| Old gate (df2c46b) | 45 of 46 (misses only the missing index, which the old gate had no rule for) |
| New gate | 46 of 46 |

Every mutation the old gate caught is caught by the new gate, including the four of R1; the new-only rules (missing index,
stale catalogue, hand-written index) are caught. The 13 snapshot-mode runs of the first review plus the 10 re-run now are
all red with a relevant finding.

### A4: Gate error paths (scratch repo, tests compiled as a dependent of the crate)

- Valid repo: clean. No `.git`: fallback walk, clean (documented as the only fallback).
- `GIT_DIR=/nonexistent`: `GateError::Tracked`, "GIT_DIR is set and `git` cannot use it: fatal: not a git repository".
  The same untracked link that passed silently before now cannot reach a verdict.
- Garbage `.git/index`: `GateError::Tracked`, "`git ls-files` failed: error: bad signature". (A truncated index that git
  itself reads as empty gives a loud `link-broken` instead; that is git's answer, not a Gate fallback.)
- `git` not on `PATH`, with and without `.git`: `GateError::Tracked`, "cannot run `git`", with the `Gate::tracked_paths`
  escape named in the message.
- `GIT_DIR` set to a valid repository: works. The real pre-commit hook of a linked worktree exports `GIT_DIR` and
  `GIT_INDEX_FILE` (probed with a throw-away hook); the real hook passed the clean state, refused a broken anchor, refused
  a frontmatter page outside the folders, and refused a link to an untracked file.
- Two new tests in `tests/gate.rs` for the failing-index and bad-`GIT_DIR` cases exist and pass.

### Gates (copy of the staged state)

| Gate | Result |
|---|---|
| `make fmt` | pass |
| `make clippy` (`-D warnings`) | pass |
| `cargo test --locked --workspace` | 1003 passed, 0 failed |
| `cargo test -p raf-workplan --release` | 263 passed, 0 failed |
| `docs_contract` (26) + `repo_gates` (57) | pass, direct and in a checkout-index snapshot |
| `make reuse` | pass |
| `make msrv` | pass |
| `make issue-refs` | pass |
| `RUSTDOCFLAGS=-Dwarnings cargo doc` | pass |
| `make docs-sync` on the staged tree | no change |

### Remaining notes (not blocking)

- A hard error when `git` is absent makes the dev-dependency use need `Gate::tracked_paths` on a host without git. This is
  fail-hard by design and the message names the remedy.
- A5 and A6 of the first review are closed by the module docs (policies listed, frontmatter rule stated) and the Gate docs.

VERDICT: PASS
