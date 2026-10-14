# Completion adversary, Stage 1 (plan audit)

- **Plan:** `.stage/shared-crates-raf-okf-raf-source/plan.md` (merged revision, 547 lines)
- **Pass:** 1. This is the single run that user decision 5 calls for. The earlier run was stopped and produced no findings.
- **Inputs read in full:** `issue.md` (title, body, 0 comments), `plan.md`, `issue-analysis.md`, `reviews/plan-review.md`, `validation.md`, `docs/capabilities/okf-profile.md`, `docs/capabilities/source-index.md`, `docs/architecture/okf-profile.md`, `work/changes/okf-profile.md` and `work/changes/source-index.md`, plus the agent rubric `plan-completion-adversary.md`.
- **Inputs read in part:** `prior-art-assessment.md` (the L1-L14 definitions), `AGENTS.md` sections 5-7, `SECURITY.md`, `Makefile` targets, `README.md`, and `crates/raf-okf/Cargo.toml` (partial code).
- **Grounds applied:** only the four plan grounds: UNFALSIFIABLE CRITERION, PRE-EMPTIVE DESCOPE, UNMAPPED REQUIREMENT and PLANNED-BY-REFUSAL. No design positions are taken.
- **Binding user decisions (not relitigated):**
  1. `tags` and `timestamp` are optional keys; `resource` stays rejected.
  2. `raf-source` has no default roots.
  3. The containment race is closed on Linux, with portable fallbacks elsewhere.
  4. Split links are refused.
  5. The port is a superset of the precedents, with licensing ignored.

  Findings F-1, F-2 and F-3 check that the plan *delivers* decisions 1, 3 and 4. None of them reopens a decision.

## Coverage

"Mapped" means the requirement has a completeness-contract row (or a design section the row defers to), an owning task, and a falsifiable check on a shipped entry point.

### Issue scope and acceptance criteria

| Requirement | Plan row / task | Status |
|---|---|---|
| Parse Markdown + YAML; `type` required; `title`/`description`/`resource`/`tags` (sorted)/`timestamp` per base OKF | rows 373, 382; decision 1; Tasks 1, 3 | **F-1** |
| Profiles: closed key set per type, typed values, unknown key a hard error | row 382, B6, Task 3 | mapped |
| Deterministic writer, round-trip stable, sorted where the spec says | row 376, B2, Task 1 | mapped (the `tags` sort is under F-1) |
| Body model: H2 byte spans, links, fences, generated and PROPOSED blocks | rows 378-380, B3, Task 2 | mapped |
| `index.md` without frontmatter is navigation | row 381, B5, Task 2 | mapped |
| `Tree` (`read`, `list`) with Disk and Memory | row 385, C1, Task 4 | mapped |
| Discovery under configurable roots (no default: decision 2); lib/bin/build.rs; rustc mod tree incl. `#[path]`; cfg(test)/`#[test]` tagged | rows 392-394, C5-C6, Task 6 | mapped |
| Test/bench/example non-shipping as `<pkg>::test_crate::<target>` etc. | row 392, C5 | **F-6** |
| Symbol index + `use`-alias table so re-exports resolve | rows 395-396, C7, Task 7 | mapped (the cfg-twins check is under F-4) |
| Port: superset of precedents | decision 5 (port), section D | mapped |
| `#[path]` cannot escape the root | row 393, Task 6 | mapped |
| AC: Memory fixtures (nested mods, `#[path]`, re-export chains, targets, rejected escape) | rows 392-396 | mapped |
| AC: OKF round-trip goldens; unknown-key fixture fails | rows 374, 376, 382; Task 1 goldens from `git ls-files` | mapped |
| AC: public APIs documented (`missing_docs` = deny) | row 399 | **F-8** |
| AC: capability docs match what was built, state → `delivered` | row 402, Task 9 | **F-7** |

### Change-record items

| Item | Plan row / task | Status |
|---|---|---|
| okf items 1-6: positioned parse, closed keys/typed/all violations, body model, writer, index, bounds | rows 373/378, 382, 378, 376, 381, 375 | mapped |
| source items 1-5: Tree, discovery, `#[path]` containment, cfg(test), symbol/alias | rows 385, 392, 393, 394, 395-396 | mapped |
| source item 6: canonical containment, non-regular, symlink, loop | row 386, Task 6 | mapped (the handle-check half is under F-2) |
| source item 7: size cap, pre-scan, dedicated stack, fuzz target | rows 387-391 | mapped (the corpus half is under F-5) |
| source item 8: file type on the opened handle, FIFO refused | row 386 | mapped |

### `okf-profile.md` Contract, Constraints and Verification

| Statement | Plan row / task | Status |
|---|---|---|
| Contract: Parse; SPDX preserved | rows 373, 374 | mapped |
| Contract: Resource bounds (1 MiB, 32, anchors/aliases/merge keys, 64; no panic or unbounded allocation) | row 375 | mapped |
| Contract: Validate (all violations, file/key/rule, typed values, `resource`, unknown `type`, one document at a time) | row 382 | mapped (the stale `tags`/`timestamp` fragment is under F-1) |
| Contract: Body model; marker errors with lines | rows 378, 380 | mapped (the Contract text is under F-7) |
| Contract: Write (deterministic, round trip, table order, comments, caller spans) | rows 374, 376 | mapped (the task mismatch is under F-4) |
| Contract: Index handling | row 381 | mapped (the Contract text is under F-7) |
| Constraints: text only, no fs or process | row 384 | mapped |
| Constraints: bounds; exact keys, no pass-through; valid or invalid; comments kept, no reflow; no position on tense | rows 375, 382, B6 (`Result<RafDocument, Vec<Violation>>`, no severity), 374, B6 future-tense negative | mapped |
| Verification: 8 bullets | rows 373, 382 (×2), 375 (×2), 378/380, 374/376, 381 | mapped |

### `source-index.md` Contract, Constraints and Verification

| Statement | Plan row / task | Status |
|---|---|---|
| Contract: Tree; nothing else opens a file | row 385, C1 `std::fs` scanner | mapped |
| Contract: Path containment; "No check separates from the open it guards" | row 386, Task 6, decision 3 | **F-2** |
| Contract: Resource bounds (cap, pre-scan budgets, 64 MiB thread, fuzz target, corpus contents, toml recursion, typed refusals) | rows 387-391 | mapped (the corpus half is under **F-5**) |
| Contract: Package discovery | row 392 | mapped (**F-6**; the text is under F-7) |
| Contract: Mod-tree following | row 393 | mapped (the false sentence is under **F-7**) |
| Contract: cfg(test) tagging | row 394 | mapped |
| Contract: Symbol index | row 395 | mapped |
| Contract: Use-alias index; consumer documents say macros are not indexed | row 396, Task 9 | mapped (**F-4**) |
| Constraints: 6 bullets | C6/row 396, 386/392, 387-390, 396, structural (`Workspace` is a snapshot), 397 | mapped |
| Verification: discovery, mod tree, `#[path]`, bounds, composites, thread name, fuzz, cfg(test), aliases, determinism, Disk = Memory, containment | rows 386-397, Task 6 | mapped |
| Verification: corpus replay asserts one of three outcomes | row 391 | **F-5** |

### Declined enhancements and descope

- The declined enhancements (19 shared Span crate, 20 `include!`, 21 reverse references, 23 `DiskTree` for docs_contract) hide no work the issue asks for.
- Decision 2 (no default roots) is the user's own scope decision, and Task 10 quotes it in the PR.
- No PRE-EMPTIVE DESCOPE was found.

## Findings

### F-1: PLANNED-BY-REFUSAL and UNMAPPED REQUIREMENT. The base keys `tags` and `timestamp` (issue scope 1, decision 1)

The issue asks for `tags` (re-emitted sorted) and `timestamp` to follow base OKF. Decision 1 accepts both as optional keys. The only completeness-contract row that names them still counts *rejecting* them as the acceptance check:

> row 382: "per type: each missing key, an extra key, `resource`, `tags`, `timestamp` -> violation with that key and `Rule`"

The plan's precedence clause does not fix this:

> line 367: "Where a row below and sections A-D differ, sections A-D and the user decisions govern."

- **No acceptance row exists.** No row maps decision 1's accepting behaviour (sorted render, duplicate tag, valid and malformed timestamp, a document without either key) to a task and a check. Only the decision text lists those fixtures.
- **The documentation edits have no task.** Decision 1 requires these edits (line 85):
  - the architecture "Document types and keys" table, its "Keys, all required" heading, and the "Relation to OKF" bullet "A document carries exactly the keys listed for its type, no more and no fewer";
  - the okf-profile Contract and Examples.

  No task names them. Task 3 (line 463) names only "the Usage/Examples doc edits; the Contract 'Validate' wording".
- **A sentence becomes false.** The okf-profile Explanation says "the key sets are closed and every key is required. A missing key then means a missing fact rather than an optional one". That is false once the keys are optional, and `change:okf-profile` does not list `#explanation` in `affects`.

`REWRITE TO:`
- In row 382, replace "`resource`, `tags`, `timestamp` -> violation" with "`resource` -> violation".
- Add the row: "Base OKF optional keys `tags` and `timestamp` (issue scope 1, decision 1) | 1 (render), 3 (validate, docs) | `Document::render` of a Capability with `tags: [b, a]` emits `tags: [a, b]` and re-renders byte-identically; `Profile::validate` returns a violation naming `tags` for `tags: [a, a]` and for `tags: [""]`, a violation naming `timestamp` for `timestamp: 2026-13-01T00:00:00Z`, and no violation for `timestamp: 2026-01-01T00:00:00Z`; a Capability with neither key validates clean; `resource:` still fails. Task 3 also:
  - rewrites the architecture 'Document types and keys' table, its heading and the 'no more and no fewer' bullet;
  - rewrites the okf-profile Contract 'Validate' and Examples;
  - rewrites the okf-profile Explanation sentences 'every key is required ... rather than an optional one';
  - first adds `cap:okf-profile#explanation` to the change record's `affects`."

### F-2: UNMAPPED REQUIREMENT and UNFALSIFIABLE CRITERION. Containment on the opened handle and `make cross-check` (Contract "No check separates from the open it guards", decision 3)

Decision 3 is how the plan makes this Contract sentence true, but nothing in the plan would fail if the post-open check were missing:

> line 251: "5. verifies containment on the opened handle, per decision 3;"
>
> line 93: "Each fallback is a plain std function compiled on every platform and unit-tested on Linux too, so the fallback code is exercised."
>
> line 95: "New gate `make cross-check`: ... It goes into `check` and gets a CI job."
>
> row 403: "| Full qualification | 10 | one `make check` (incl. `test-release` and `fuzz`) |"

- **The new check is never exercised.** Every containment fixture in row 386 and Task 6 is refused at step 2 (canonical prefix) or step 4 (`fstat`). No named fixture exercises step 5, so removing the Linux `/proc/self/fd` check, the other-unix `(dev, ino)` check or the non-unix re-canonicalisation fails nothing.
- **The fallback tests are unspecified.** "Unit-tested on Linux too" names no input and no observation.
- **The new gate has no owner.**
  - `make cross-check` has no task. Task 4 and the Task 5 gate-wiring commit omit it, and row 403 leaves it out.
  - The C4 gate-list update (Makefile header, AGENTS.md section 3, CONTRIBUTING.md, PR template, CHANGELOG Gates entry) does not include it.
- **The SECURITY.md bullet has no owner.** Decision 3 requires the per-platform statement in the SECURITY.md Containment bullet, but Task 5 owns only Bounds and Task 9 only present-tense text.
- **The Task 9 checklist has nothing to cite.** It has no test to cite for "No check separates from the open it guards".

`REWRITE TO:` Add the row: "Handle containment per platform (source-index Contract 'No check separates from the open it guards', decision 3) | 4 (code, fixtures, Contract and SECURITY.md Containment bullet), 5 gate commit (`make cross-check`) | Fixtures, all run on Linux:
- a fixture swaps an intermediate directory for a symlink to outside the root between the prefix check and the open (through a seam in `tree/disk`), and `DiskTree::read` returns `OutsideRoot` with the sentinel unread;
- the Linux verifier, given a handle on a file outside the root, returns `OutsideRoot`, and an in-root neighbour returns `Ok`;
- the other-unix verifier, given a path whose re-canonicalised `(dev, ino)` differs from the handle's, returns `OutsideRoot`, and an equal pair returns `Ok`;
- the non-unix verifier refuses a non-regular handle with `NotRegular` and an outside re-canonicalised path with `OutsideRoot`, each beside an accepted case.

Gate:
- `make cross-check` exits 0 for `x86_64-apple-darwin` and `x86_64-pc-windows-gnu`, and exits non-zero naming the target when that target is not installed;
- it is in `check` with a CI job, and is named in the Makefile header, AGENTS.md section 3, CONTRIBUTING.md, the PR template and the CHANGELOG Gates entry;
- row 403 reads 'incl. `test-release`, `fuzz` and `cross-check`'."

### F-3: UNMAPPED REQUIREMENT. Split links in repository tooling (decision 4)

> line 99: "Through `Body::parse`, `docs_contract` applies the rule to every tracked Markdown file."
>
> line 100: "The architecture sentence becomes 'A link split across lines is a violation.'"
>
> Task 8 (lines 519-520): "docs_contract uses `Document::parse`, `Body::parse` (every frontmatter-less tracked Markdown file), `Profile::validate`, ... It keeps only these: cross-document checks, link resolution against tracked files, ..."

- **No completeness row.** Decision 4 has no completeness-contract row.
- **No docs_contract fixture.** For frontmatter-less files, `Rule::SplitLink` belongs to `Profile` (B6), not to `Body::parse`. Task 8 does not say that docs_contract reports it, and no fixture shows docs_contract failing on a split link in a file without frontmatter.
- **No task for the sentence edit.** The architecture sentence edit has no task.

`REWRITE TO:` Add the row: "Split links refused (decision 4) | 2 (`Body` records, architecture sentence, rejoin), 3 (`Rule::SplitLink`), 8 (docs_contract) | the four decision-4 fixtures through `Body::parse` and `Profile::validate`; docs_contract fails on a frontmatter-less tracked-file fixture holding `[a](\nb.md)` and naming its line, and passes on the one-line neighbour; Task 2 rewrites the architecture 'Files and repository rules' sentence to 'A link split across lines is a violation.'"

### F-4: UNFALSIFIABLE CRITERION. Completeness rows that contradict the design they check

> row 396: "cfg twins -> `Ambiguous`"
>
> C7 line 338: "Same path, namespace and kind gives one symbol with several sites (cfg twins)."
>
> Task 7 line 504: "cfg twins as a multi-site symbol;"

- **Row 396.** Whichever behaviour is built, one of the plan's own checks passes and the other fails. The line-367 precedence clause leaves the acceptance check open to reinterpretation.
- **Row 376.** It assigns "`replace_span` changes only that span and the body model is rebuilt so a second splice uses fresh spans" to Task 1. The splice API is `Body::replace_spans` in B4, which is Task 2, so the check cannot run where the row places it.

`REWRITE TO:`
- In row 396, replace "cfg twins -> `Ambiguous`" with "`#[cfg(unix)] fn a()` beside `#[cfg(windows)] fn a()` -> `Ok(Resolution)` with exactly two sorted `sites` and `via_glob == false`; two globs exporting `x` -> `Ambiguous`".
- In row 376, set the task to "1 (render), 2 (splice)", and change the splice check to "`Body::replace_spans` with two generated blocks changes only those byte ranges; a second call uses spans read from the rebuilt model; an overlapping pair returns `SpliceError::Overlap` and the body is unchanged".

### F-5: UNFALSIFIABLE CRITERION. Corpus contents and replay outcomes (source-index Contract "Its corpus ... holds ...", Verification "Corpus replay")

> row 391: "`fuzz/fuzz_targets/parse_source.rs` exists; replay test iterates every corpus file in a child; `make fuzz` exits 0"
>
> C4 line 291: "A replay test runs every corpus file in a child process."
>
> C4 line 281: "The corpus holds the shapes and the at/over-budget files."

- **The outcome assertion is missing.** The Verification bullet requires the replay test to assert "that each ends in one of the three outcomes above". The row and C4 only require it to *run* each file, so a replay that observes any exit passes.
- **The corpus contents are unchecked.** The Contract says the committed corpus "holds the composite and single-shape fixtures of Verification below, files at and just over each budget". No check confirms this, so an empty or partial corpus passes row 391 vacuously.

`REWRITE TO:` In row 391, say: "the replay test:
- asserts that `fuzz/corpus/parse_source/` contains a named file for each of the 5 single shapes, 5 composites and 6 uncounted shapes, and an at-budget and an over-budget file for each of the 5 budgets, and fails on a missing name;
- runs every corpus file in a child process, in debug and in release;
- asserts each file ends in exactly one of three outcomes: a typed refusal naming the file and budget; exit 0 with the index equal to the expected symbol list the test records for that file; or a non-zero abort;
- has a negative fixture: a recorded list that differs from the real index makes the replay test fail."

### F-6: UNFALSIFIABLE CRITERION. The target crate paths named by the issue (issue scope "`<pkg>::test_crate::<target>`, etc.")

> row 392: "`MemoryTree` fixture per target kind x (default, explicit path) asserting kind, name and `shipping`"
>
> C5 line 298: "`Target { package (Cargo name), name (Cargo target name), kind, crate_path, root, shipping }`"

The issue names the path format of non-shipping targets, and C5 builds `crate_path` (`bin_crate`, `test_crate`, `bench_crate`, `example_crate`, `build_script`). The row asserts only the Cargo target name, and no fixture asserts that an item in a test, bench or example target resolves under its crate path. A wrong or missing `crate_path` passes row 392.

`REWRITE TO:` In row 392, change the check to "asserting kind, Cargo target name, `shipping` and `crate_path`:
- the lib is `demo`;
- the binary is `demo::bin_crate::x`;
- the integration test is `demo::test_crate::it`;
- the bench is `demo::bench_crate::b`;
- the example is `demo::example_crate::e`;
- the build script is `demo::build_script`.

For each target, an item resolves under that path. For example, `ws.resolve("demo::test_crate::it::case")` returns a site in `tests/it.rs` whose target has `shipping == false`."

### F-7: UNMAPPED REQUIREMENT and UNFALSIFIABLE CRITERION. "Docs updated to match what was built" (issue AC 4, I-24)

> section A line 133: "**Docs stay current per commit.** AGENTS.md members and layout change in Tasks 1 and 4. SECURITY.md Bounds changes in Task 5. Task 9 carries only state-dependent text."
>
> row 402: "AGENTS.md, README.md, architecture page, SECURITY.md and CHANGELOG.md state the present"
>
> Task 9 line 538: "A checklist mapping every Contract, Constraints and Verification sentence to its executing test is recorded in `validation.md`."

**Contract sections with no owning task.** Capability Contract edits are assigned only to Task 3 ("Validate"), Task 4 ("Tree"/"Path containment"), C3 (counter definitions) and C5 (skipped directories). No task owns these:

- **A sentence the planned build contradicts.** source-index "Mod-tree following": "relative to the directory the module would otherwise use". C6 builds rustc's rule, under which a `#[path]` in the non-mod-rs `src/a.rs` resolves against `src/`, not `src/a/`.
- **A sentence the planned build contradicts.** source-index "Package discovery": "finds every package". C5 line 295 stops: "It does not descend below a `[package]` manifest that lacks `[workspace]`."
- **Unstated built behaviour.** The same section states neither the build-script target nor the `<pkg>::*_crate::<target>` paths, both of which the issue requires.
- **Unstated built behaviour.** source-index "Use-alias index" is silent on `Resolution` and cfg twins.
- **Text that omits what the issue requires.** okf "Body model" gives only a "line span" (the issue requires byte spans), names no `Body::parse` and no link forms. okf "Index handling" names no `Index::parse`.

**Why the planned checks miss these.**
- The Task 9 checklist runs one way only: each document sentence to a test. It does not catch built behaviour that the documents omit.
- Row 402's "state the present" names no lines.

`REWRITE TO:` Assign the Contract edits to tasks:
- **Task 2:** okf "Body model" (line and byte spans, `Body::parse` for Markdown without frontmatter, link forms, split links refused) and "Index handling" (`Index::parse`, `is_capability_index`).
- **Task 6:** source-index "Package discovery":
  - explicit roots;
  - the recursion and stop rule;
  - skipped `target` and dot directories;
  - the build-script kind and its shipping flag;
  - the `<pkg>::{bin,test,bench,example}_crate::<target>` and `<pkg>::build_script` paths.
- **Task 6:** source-index "Mod-tree following":
  - replace "relative to the directory the module would otherwise use" with rustc's rule (the declaring file's directory plus the inline-module components);
  - a `#[path]` file is mod-rs for its children;
  - `name.rs` beside `name/mod.rs` is an error;
  - `cfg_attr` path alternatives;
  - a module cycle is an error.
- **Task 7:** source-index "Use-alias index" (the `Resolution` fields; cfg twins as one multi-site symbol; `Ambiguous` only for glob against glob or two targets).

Then make row 402's check read: "the Task 9 checklist runs both ways, with no unmatched entry in either direction:
- every Contract, Constraints and Verification sentence maps to a test whose assertion matches its wording;
- every completeness-contract row maps to the Contract sentence that states its behaviour.

AGENTS.md 'Every crate below is **designed, not yet implemented**' and '(none exist yet; `members` is empty)', README.md 'designed — not yet published' for `raf-okf` and `raf-source`, the architecture page 'binding waits for the source index' and 'will enforce the whole profile once they are implemented', and SECURITY.md 'Once implemented' for the two delivered surfaces are each replaced. The Task 9 review confirms by grep that none of those strings remain for these two crates."

### F-8: UNFALSIFIABLE CRITERION. `missing_docs` = deny (issue AC 3)

> row 399: "`cargo clippy -p <crate> --all-targets -- -D warnings` and `RUSTDOCFLAGS=-Dwarnings cargo doc -p <crate> --no-deps`"

`missing_docs` is allow-by-default, and `-D warnings` does not raise an allowed lint. Both commands pass on an undocumented public API unless the crate inherits `[workspace.lints]` through `[lints] workspace = true`, and no check asserts that inheritance for `raf-okf` or `raf-source`. The plan does assert it for the fuzz manifest (C4, line 278).

`REWRITE TO:` Add to row 399: "a `repo_gates` test asserts that every `[workspace] members` manifest carries `[lints] workspace = true`, with a negative fixture of a member manifest without it failing, alongside the clippy and doc commands."

## Scope decisions

- None of these findings needs a new user decision. Each one is a plan-text edit that maps or makes falsifiable a requirement already in scope.
- F-1, F-2 and F-3 implement decisions 1, 3 and 4 as the user stated them.
- The other decisions are respected as written: decision 2 (no default roots, quoted in the PR) and decision 5 (the superset port, and this single run).

## Result

FINDINGS-OPEN: F-1, F-2, F-3, F-4, F-5, F-6, F-7 and F-8 are open. Each has a `REWRITE TO:` line above, and no user decision is required.

VERDICT: NEEDS-REVISION

## Recheck (pass 2)

- **Plan re-read:** the revised `plan.md` (566 lines), in full.
- **Scope:** F-1..F-8, plus the new text their fixes introduced:
  - completeness rows 376, 382, 383, 392, 393, 397, 400, 403, 404, 405 and 406;
  - the "Docs:" and "Includes" lines added to Tasks 2-7;
  - the "Completion adversary (pass 1)" section.
- **Not reopened:** unchanged text cleared in pass 1.

| Finding | Status | Evidence in the revised plan |
|---|---|---|
| F-1 | DISCHARGED | **Row 382** now reads "each missing key, an extra key, `resource` -> violation"; the refusal of `tags`/`timestamp` is gone. **New row 383** ("Base OKF optional keys `tags` and `timestamp`", Tasks 1 and 3) checks: sorted render that re-renders byte-identically, `tags: [a, a]` and `tags: [""]` refused, a malformed timestamp refused and a valid one accepted, a document with neither key clean, and `resource:` still failing. **Task 3 Docs (line 475)** first adds `cap:okf-profile#explanation` to `affects`, then rewrites the architecture table, its heading and the "no more and no fewer" bullet, the okf Contract "Validate" and Examples, and the Explanation sentences. |
| F-2 | DISCHARGED | **New row 404** ("Handle containment per platform") checks: a seam swap between the prefix check and the open gives `OutsideRoot` with the sentinel unread; each per-platform verifier is refused beside an accepted case, all run on Linux; `make cross-check` passes for both targets, fails naming a missing target, sits in `check` and CI, and appears on every gate list. **Task 4 (line 482)** owns the verifiers, the seam fixtures and the SECURITY.md Containment bullet. **Task 5 (line 489)** owns the `make cross-check` gate. **Row 406** names `cross-check`. |
| F-3 | DISCHARGED | **New row 405** ("Split links refused", Tasks 2, 3 and 8) checks the four decision-4 fixtures through `Body::parse` and `Profile::validate`, and a docs_contract fixture without frontmatter that holds `[a](\nb.md)`: the check fails naming its line, and the one-line neighbour passes. **Task 2 Docs (line 468)** owns the architecture sentence and the rejoin. |
| F-4 | DISCHARGED | **Row 397** now expects `#[cfg(unix)] fn a()` beside `#[cfg(windows)] fn a()` to give `Ok(Resolution)` with exactly two sorted `sites` and `via_glob == false`, which agrees with C7 line 338 and Task 7 line 521. **Row 376** is "1 (render), 2 (splice)", with the `Body::replace_spans` check: only those byte ranges change, a second call uses rebuilt spans, and an overlap returns `SpliceError::Overlap` with the body unchanged. |
| F-5 | DISCHARGED | **Row 392** now requires the replay test to assert a named corpus file for each of the 5 single shapes, 5 composites and 6 uncounted shapes, plus an at-budget and an over-budget file for each of the 5 budgets. It runs every file in a child process in debug and release, and asserts exactly one of the three outcomes against a recorded symbol list, with a negative fixture. **C4 line 291** is less specific than the row but does not contradict it. |
| F-6 | DISCHARGED | **Row 393** asserts `crate_path` for all six target kinds (`demo`, `demo::bin_crate::x`, `demo::test_crate::it`, `demo::bench_crate::b`, `demo::example_crate::e`, `demo::build_script`), and that an item resolves under each, e.g. `demo::test_crate::it::case` with a site in `tests/it.rs` and `shipping == false`. |
| F-7 | DISCHARGED | **Task 2 Docs (468)** owns okf "Body model" and "Index handling". **Task 6 Docs (511)** owns "Package discovery", including the stop rule, the build script and the `*_crate` paths, and "Mod-tree following", replacing "relative to the directory the module would otherwise use". **Task 7 Docs (531)** owns "Use-alias index" and "Symbol index". **Row 403** requires the two-way checklist with no unmatched entry, and replacing the named stale AGENTS.md, README.md, architecture and SECURITY.md strings, confirmed by grep. |
| F-8 | DISCHARGED | **Row 400** adds a `repo_gates` test asserting that every `[workspace] members` manifest carries `[lints] workspace = true`, with a negative fixture. |

**New text.** I found no new UNFALSIFIABLE CRITERION, PRE-EMPTIVE DESCOPE, UNMAPPED REQUIREMENT or PLANNED-BY-REFUSAL. Three advisory nits, none of them material because each row named governs acceptance:

- **Checklist wording.** Task 9 line 557 and Verification line 439 still describe a one-way "sentence-to-test" checklist; row 403 requires it both ways. Say "both ways, per row 403" in both places so the implementer and the Stage 2 audit read the same requirement.
- **Duplicate entry.** Row 397 lists "two globs -> `Ambiguous`" twice. Drop one.
- **Stale label.** The requirement column of row 376 still says `replace_span`; the check column correctly names `Body::replace_spans`.

Completion rubric, pass 2: UNFALSIFIABLE CRITERION none; PRE-EMPTIVE DESCOPE none; UNMAPPED REQUIREMENT none; PLANNED-BY-REFUSAL none. User decisions 1-5 are respected as written.

PASS

VERDICT: PASS
