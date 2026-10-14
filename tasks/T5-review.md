# T5 review: generated blocks, sync and write

Scope: the staged, uncommitted change on top of 6970f8a (`git diff --cached`, 19 files):
`generated.rs`, `sync.rs`, `examples/docs_sync.rs`, the load and model changes, the six new
finding kinds, the tests (`generated.rs`, `sync.rs`, `support/synced.rs`, `support/cases.rs`),
Makefile, CHANGELOG, AGENTS.md. Binding specs: `workplan-generated-docs.md`, `okf-profile.md`.
Plan: Task 5, G6, G10-G13, G21, G23, R14/R15/R20/R27.

All experiments ran in `/tmp/t5/wt` (a copy, own `CARGO_TARGET_DIR`, `CARGO_BUILD_JOBS=6`).
Nothing in the worktree was changed except this file. One slip, undone: a `git add -N` run in
the copy wrote an intent-to-add entry for `docs/capabilities/index.md` into the shared git
index (the copy's `.git` points at the real gitdir). I removed it with `git reset -q --
docs/capabilities/index.md`; the staged state is again 19 files, 3309 insertions, no unstaged
change.

## Gates on the staged state (copy)

| Gate | Result |
|---|---|
| `cargo test -p raf-workplan` (debug, incl. doctests and the `docs_sync` example run) | pass, 16 targets |
| `cargo test --release -p raf-workplan` | pass |
| `cargo clippy --workspace --all-targets -- -D warnings` | pass |
| `cargo fmt --all -- --check` | pass |
| `docs_contract` + `repo_gates` | pass |
| `make reuse` | pass; also pass with the `index.md` that `make docs-sync` creates (313/313 files) |
| `make msrv` (1.98.1) | pass |
| `make doc` (rustdoc -Dwarnings), `make deny` | pass |
| `make docs-sync` twice on a copy of this repository | 1st: `rewrote docs/capabilities/index.md`; 2nd: nothing, exit 0 |

No new dependency (Cargo.toml and Cargo.lock untouched), no `unsafe`, no `#![feature]`. The
added lines hold no issue or pull request number, plan wording or attribution (grep of the
diff). SPDX headers are present on every new file. The two SPDX tag names in the index renderer
are joined with `concat!`, so the source carries no tag of its own.

## 1. Spec statement by statement

| Statement | Built | Tested |
|---|---|---|
| Block syntax, open once, close once, no nesting | raf-okf (earlier task) | marker fixture: nested, unclosed, orphan end, non-kebab kind |
| Unknown kind is an error | `generated-unknown-kind`, sync refuses | marker fixture, case in `kinds` |
| `verification-matrix`: one row per tests/examples binding, kind, sections, sorted by path | `matrix()` | exact bytes (`assert_eq!`), sort test, header-only case |
| `relationships`: depends on, depended on by, changed by, links | `relationships()` | exact bytes for three capabilities, header-only case |
| `catalogue`: group order, slug order, gaps for implementing, link and description | `catalogue()` | exact bytes of the whole index; slug-order test; gap test |
| Index: no frontmatter, only blocks, hand-written index rejected | `Index::parse` plus `index-hand-written` | three inputs, incl. text after a header |
| Sync pure, same bytes, no timestamps, stable order | `sync()` returns text; sorted by path | `the_report_is_a_pure_function...`, twice-sync test |
| Line ending of the body (G13, G21) | `line_ending`, `Eol`, `mixed-line-endings` | CRLF, LF, CRLF head over LF body, LF head over CRLF body, mixed |
| File with no block untouched; text outside never changed | `file()` returns `None`; head kept byte for byte | `text_outside_a_block...`, frontmatter byte test |
| Stale names file and kind; sync repairs | message names both; one pass behind check and sync | edited/removed/added per kind, `assert_stale_and_repaired` |
| Editing inside a block makes it stale | same | edited-row cases |
| Placement (G11); blocks do not count toward non-empty | `misplaced()` | five placement fixtures, `okf-section-empty` fixture |
| Idempotence | | twice/three-times test (but see R1) |
| `preview` | Task 8 | n/a, correctly absent |

Exact-output fixtures assert bytes: matrix, relationships, catalogue and the new index are
`assert_eq!` on whole strings. Weaker (`contains`) and acceptable: the unresolved-code gap
text, the escape rule row, the added-row cases.

## 2. check and sync agreement (adversarial)

Each case: `check`, then `sync`, apply, `check` again, `sync` again.

* Trailing space on a row, double trailing space, an extra blank line inside a block: stale,
  repaired, second sync empty.
* Two blocks of one kind in one document: both repaired, second sync empty.
* Marker lines inside a code fence, inside a PROPOSED quote, inside a plain quote: not read as
  blocks, not rewritten, no forgery. In a list item the end marker is an orphan: a reader error,
  `sync` returns `Unloaded`.
* Binding paths with `|`, a backtick, `\n`, `\r\n`, a backslash, `-->`, a full end marker, a
  full open marker (as unresolved bindings): the markers are not forged (`<` becomes `&lt;`),
  `|` becomes `\|`, newline becomes a space, sync of the result is empty.
* Descriptions with newline, pipe, backtick, `## Heading`, a full end marker, `\r\n`, `\r`, a
  fence, a link: one row each, marker not forged, second sync empty.
* An existing CRLF index with a stale row: repaired in CRLF, no bare LF.
* Sync repairs in all of these; `check` is clean afterwards (apart from the binding findings
  the fixtures intentionally create).

Not clean, see R1: a block in a section other than Verification.

## 3. write safety (experiments)

* `RelPath` refuses an absolute path and a path that climbs above the root. `docs/../docs/...`
  stays inside and is accepted.
* Parent directory that is a symlink out of the root: refused (`OutsideRoot`), outside file
  untouched. Parent symlink inside the root: written through. Symlinked target and dangling
  index symlink: refused.
* Read-only file in a writable directory: replaced by rename, mode kept (0444). Read-only
  directory: `PermissionDenied`, no temp file left.
* Report of three edits where the 2nd fails (read-only directory): the 1st is written, the
  2nd and 3rd are not. **"Nothing is written if any edit fails" holds for compute failures and
  refusals judged before the first write, not for I/O failures.** The `write` rustdoc says so
  and says a second sync completes the rest, so the contract is honestly stated and the
  recovery path (idempotent sync) exists. Plan G6 says "fails to compute", which is met.
* A file that already exists at `.<name>.sync.tmp` makes `create_new` fail, and the failure
  branch then `remove_file`s it: the user's file is deleted (confirmed).
* The same file listed twice as `x` and `docs/../docs/x` bypasses the duplicate check
  (harmless, the last rename wins).
* Between `judge` and `replace` the path is re-resolved by `std::fs` (not the contained tree):
  a directory swapped for a symlink in that window is not caught. A local-race limit on a
  developer's own checkout.

## 4. Mutation (re-run)

| Mutation | Result |
|---|---|
| drop matrix sort | `a_verification_matrix_lists...`, `rows_are_sorted_by_path...` fail |
| `|` not escaped | `cell_text_is_escaped...`, catalogue exact test, `the_roadmap_rows_equal...` fail |
| `<` not escaped | `cell_text_is_escaped...`, `the_catalogue_cannot_be_forged...` fail |
| catalogue group order reversed | catalogue exact test, two roadmap-equality tests fail |
| CRLF written as LF | three CRLF tests fail |
| catalogue accepted anywhere | `a_catalogue_block_is_valid_only...`, `every_kind_is_produced...` fail |
| every block always stale | about 12 tests fail (synced base clean, idempotence, repairs) |
| unknown kind accepted | marker fixture and `every_kind_is_produced...` fail |
| mixed line endings ignored | `a_body_that_mixes...` and the kind coverage test fail |
| relationship sort reversed | `a_relationship_table_lists...` fails |
| drop `dedup` of relationship rows | **survives** (no fixture repeats an edge) |
| drop relationship sort | **survives** (the graph already yields slug order; equivalent in practice) |

## Decisions judged

(a) `index-hand-written` replacing `okf-parse-index-text`: sound. It is the spec's wording, the
coverage test excludes the reader kind and counts one fewer, and the ported index-text case now
names the new kind. Accept.

(b) `index-missing` only with a capability document: sound; a repository with none needs no
catalogue, tested (`a_repository_without_capabilities_needs_no_index`).

(c) `Unloaded` while any capability or change document fails: sound and fail-hard; tested for
a bad state and a bad type. It also stops a document with a bad marker from being synced
around.

(d) Filtering `index-missing` and `generated-stale` from the shared fixtures: acceptable. The
shared fixtures hold no generated block and no index, so those two kinds can only be the
"not synced" noise, never a regression the filter hides. The generated rules are tested on a
synced base in `generated.rs`/`sync.rs`, and `generated_cases()` (unfiltered) feeds the
kind-coverage and check-iff-sync tests. The residual risk is a false `generated-stale` in a
repository that has blocks but is not one of the synced fixtures, which the
`check_is_clean_exactly_when_sync_has_nothing_to_write` loop over all cases does cover. Accept.

(e) Rendering choices: the relation label repeated per row matches the spec example. An
unresolved example labelled `example target` is the right choice but has no fixture (A3). The
escape rule is documented in the module and in lib docs, and is unit-tested, with `<` for
marker safety (verified). Backtick in a path is not handled inside the code span (A4).

(f) `git ls-files` only with `.git`: fine (a worktree `.git` file counts via `exists()`);
only link findings depend on it and sync does not read them.

## Findings

### REQUIRED

**R1. Sync is not idempotent when a generated block sits in a section other than
Verification, and the matrix then substantiates itself.** `graph::prose_of` (section text for
`substantiated`) does not blank generated blocks, so the rows a matrix writes
(`` `demo::test_crate::it::case` `` ...) count as a code span naming the path in the section
that holds the block. Repro (fixture `unsynced()` with a `verification-matrix` block placed in
gamma's Usage section after the line naming the example): round 0 writes the block, round 1
rewrites gamma again, round 2 is empty. `check` after the first sync reports
`generated-stale` for gamma, and the converged table is wrong: it lists `[Usage](#usage)`
for `demo::real` (named nowhere except in the block) and adds Usage to the test. A matrix in
Contract behaves the same. This breaks the spec Constraint "Sync is idempotent: running it
twice changes nothing the second time" and the Contract "a pure function of its inputs"
(the output depends on its own previous output), and G11 explicitly allows a block in any
section. Fix, verified in the copy: in `prose_of`, blank the generated blocks as well:
`.chain(body.generated_blocks().iter().map(raf_okf::GeneratedBlock::outer))`. With it the
repro converges in one sync with the correct sections (`[Contract]`), and the full
raf-workplan suite still passes. Add a fixture: a matrix in Usage and in Contract, assert one
sync leaves `sync()` empty and `check` clean and the sections are those of the prose. This also
corrects `tests_for(cap, section)` and the section data in the graph.

### ADVISORY

**A1. Partial application on an I/O failure.** `write` judges every target first, then
renames one by one, so a failing 2nd of 3 leaves the 1st written (verified). Documented in the
rustdoc and recoverable by re-running sync, and G6 only promises "nothing is written if any
edit fails to compute". A cheap tightening is to write all temporary files first and rename
afterwards, which turns disk-full and permission failures on creation into all-or-nothing. If
not done, say so in the CHANGELOG entry and in the `docs_sync` module doc, which today print
nothing for the files already written when a later write fails.

**A2. The failure branch deletes a pre-existing `.<name>.sync.tmp`** that `create_new` refused
to overwrite (verified: a "precious" file was removed). Clean up only when this call created
the file. Also, a leftover from a killed run sits in a directory the document layout treats as
Markdown-only; the cleanup-on-next-run is the useful half of this behaviour, so keep that by
tracking whether the file was created.

**A3. Two behaviours with no fixture:** an unresolved example listed as `example target`
(decision e), and the relationship `dedup`. The `a_path_named_in_a_fence_or_a_proposal...` test
covers only the fence half in its body. Add the proposal case.

**A4. A backtick in a binding path breaks the code span of the matrix cell** (`` `x`y` ``). The
path is also reported as unresolved, so this is cosmetic. Use a delimiter longer than the longest
backtick run in the path.

**A5. `docs_sync --check` cannot tell stale from failure by exit status** (both 1; stdout vs
stderr differ), and it demands `--copyright`/`--licence` even though they only matter when the
index is missing. Both are consistent with the module doc and the plan ("exit codes beyond 0
and 1" are out of scope), so this is a note for Task 7/CI wiring.

**A6. `the_catalogue_of_this_repository_equals_its_roadmap` loads the source with roots
`["crates"]`**, while the example and G7 use `["."]`. The test compares two views of one plan, so
it holds either way, but loading `["."]` would match the shipped example.

**A7. Wording still to land in Task 9 (list U):** the Contract line "with `\n` line endings"
and the `cargo workplan docs sync` Usage block are unchanged in the spec, as the plan says.
The code and tests already implement the reworded form (G13, G21).

## Verdict reasoning

Everything specified is built, byte-tested and mutation-sensitive, and all nine gates pass.
One defect breaks a stated constraint (idempotence) for a placement the plan allows, has a
one-line verified fix and needs a regression fixture.

VERDICT: BLOCKED

## Recheck

Run on a copy with its own git dir (`/tmp/t5/wt2`, `git init` of the staged tree; nothing run
against the shared index this time). Staged state: 20 files, 3567 insertions.

**R1, closed.**
* Own repro (the earlier adversarial harness): a `verification-matrix` in gamma's Usage and in
  its Contract. Before: round 1 rewrote gamma again and the table listed Usage wrongly. Now one
  sync gives `check` clean and a second `sync` empty, and the sections are those of the prose.
* Mutant: removing the `generated_blocks` chain from `graph::prose_of` fails exactly
  `a_block_in_a_section_other_than_verification_converges_in_one_sync`; restored, all pass.
* The fix is the one-line blanking I proposed. The regression covers a matrix and a
  relationships block in Usage.

**Adversarial check and sync cases, re-run.** All the earlier cases (trailing whitespace, extra
blank line, two blocks of one kind, markers in a fence, in a PROPOSED quote, in a plain quote,
paths and descriptions with pipe, backtick, newlines, backslash, `-->` and full markers, CRLF
index) give 38 of 38 empty second syncs; the one remaining case (marker in a list item) is an
orphan-end reader error and `sync` returns `Unloaded`, as before. No marker is forged. A path
with a backtick now renders with a longer fence (A4).

**Write safety probes.**
* Staging failure (2nd of 3 targets in a read-only directory): `Permission denied`, the 1st and
  3rd targets unchanged, no temporary file left. This is the A1 behaviour.
* Rename failure midway: used a sticky directory (`/dev/shm`) with a file owned by root, so the
  2nd rename returns `EPERM` after the 1st succeeded. Result: the 1st target holds the new
  text, the 3rd is unchanged, the temporary files of the 2nd and 3rd are gone, no litter. This
  is exactly the remainder the new rustdoc describes (renames already done stay, sync again
  completes). The root-owned file was given its own content (empty), so nothing could be
  damaged.
* Temp-name collision: decoys at the old fixed name and at 300 consecutive
  `.<name>.<pid>.<n>.sync.tmp` names: the write succeeds, every decoy intact. Temp names that
  are symlinks to an outside file are not followed (`create_new` fails and the loop moves on);
  the outside file is unchanged.
* Mutant: renaming each file as it is staged (the old per-file behaviour) fails the atomic
  write test and the other write tests.

**Dedup removal, confirmed dead.** `generated::relationships` filters `graph.depends()` and
`graph.changes_edges()` on one end of an edge and collects the other. Both vectors are built
only in `graph::build`, which sorts them by `(from, to)` / `(change, capability)` and calls
`dedup` on the whole edge before the graph is returned (`graph.rs` 730-738); their fields are
private and `Graph::default()` is empty. For a fixed `id` the collected ends are therefore
distinct. The new fixture `a_relation_listed_twice_is_one_row` names a dependency twice and a
change listing twice in the documents and gets one row each, so the guarantee is now pinned from
the document side. Removing the call was right.

**A-items.** A1 done (stage all, rename after; rustdoc states the rename remainder), A2 done
(unique exclusive names, only own files removed), A3 done (fixtures for the unresolved example
and the repeated relation), A4 done, A6 done (roots `["."]`). A5: exit codes 0 / 1 (stale under
`--check`) / 2 (any failure) and the SPDX flags only when the index must be created, verified by
running the example (`--check` without flags on a repository with no index exits 2 with a
message naming `--copyright`; with flags it prints `stale docs/capabilities/index.md` and exits
1; after a sync, `--check` with no flags exits 0).

**Gates on the staged state (copy).** rustfmt, workspace clippy `-D warnings`,
`cargo test -p raf-workplan` (debug, 16 targets all ok, and release), `docs_contract` plus
`repo_gates`, `make reuse` (312/312), `make msrv`: all pass.

**New advisory notes (not blocking).**
* `write_never_touches_a_file_...under_a_temporary_name` plants its decoy at counter 0 of a
  process-wide counter, so another test running first can use that number and the collision
  branch is not guaranteed to be exercised. Plant decoys for a range of counters, as in the
  probe above.
* `docs_sync --check` on a repository whose index is missing and without the SPDX flags exits
  2 rather than 1, although "the index is missing" is the stale case. It still fails and says
  why; accepted.
* The window between the judge and the renames (a parent directory swapped for a symlink)
  remains, as noted before; not a regression.

All previous REQUIRED and ADVISORY items are resolved or accepted.

VERDICT: PASS
