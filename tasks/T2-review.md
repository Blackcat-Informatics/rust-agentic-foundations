# T2 review: git module, Gate::open/programs, root discovery, shared change lookup

## Scope
Staged diff of Task 2 on top of fefadb2 (20 files: src/git.rs, src/git/tests.rs, src/root.rs, src/root/json.rs and
tests, src/lookup.rs, gate.rs, preview.rs, exec.rs and exec/invocation.rs additions, lib.rs, examples/docs_sync.rs,
tests/{git,root,gate,sync,preview,exec}.rs, CHANGELOG.md), read against plan D3, D4, D7, Task 2 and the Contract-edit
table, AGENTS.md, SECURITY.md, workplan-reconcile.md (Invocation, Merge base), workplan-cli.md (Root discovery) and the
delivered document-model and generated-docs text. Experiments ran in throwaway worktrees (HEAD, and HEAD plus the
staged diff), both removed afterwards together with their target directories. Nothing was added, reset, stashed or
committed in the task worktree; `git status` is unchanged.

## Evidence

Gates (task worktree, CARGO_BUILD_JOBS=6, all exit 0): `cargo test -p raf-workplan` 412 passed, 0 failed;
the same with `--release` 412 passed; workspace clippy `--all-targets -D warnings` clean; `cargo doc
-Dwarnings` clean; `cargo fmt --check` clean; `docs_contract` + `repo_gates` pass (57 in the last binary,
includes issue-refs); `make reuse` compliant (331/331); `make deny` "advisories ok, bans ok, licenses ok, sources ok".

### (1) Behaviour preservation of Gate and the example
* Read the old `git_tracked` against the new `git::tracked_paths` line by line: same two-step shape (`rev-parse
  --is-inside-work-tree` then `ls-files -z`), `None` only for "false" or a failure containing "not a git repository"
  with neither GIT_DIR nor GIT_WORK_TREE set, GIT_DIR/GIT_WORK_TREE failure is an error naming the variable,
  UTF-8 and `RelPath` checks identical, symlink filtering and `LinkContext` choice identical. `Gate::open` keeps the
  old order (DiskTree, capability check, Workspace::load, tracked list); `try_check` is `open()?.workplan().check()`.
* HEAD's own `tests/gate.rs`, `tests/sync.rs` and `tests/preview.rs` (copied from HEAD over the staged ones) pass
  unchanged against the new library (13 + 20 + 17... all green). In the staged diff the pre-existing tests lose only
  import lines; everything else is added.
* Example: built HEAD's `docs_sync` and the staged one and ran both on scratch roots.
  - root with Cargo.toml + src and no `docs/capabilities`: HEAD exit 0, no output; staged exit 2 with the gate's
    message ("docs/capabilities does not exist; a repository with no capability document ... refuses it").
  - root with an empty `docs/capabilities`, `--copyright/--licence` given: HEAD exit 0 and writes nothing (no
    index); staged exit 2 ("holds no capability document").
  - Both exits are as the implementer reported. Normal roots (documents present) behave identically; the existing
    example tests pass.
  Judgement: this does not contradict the delivered generated-docs text. That text ("A missing index is an error
  when there is a capability", "make docs-sync", "--check exit 1 when stale", SPDX flags read only when the index
  must be created) says nothing about a root with no capability document, and `make docs-sync` always runs on this
  repository, which has documents. The new behaviour agrees with the document-model Gate text (no capability
  document is an error, not a clean verdict). It is disclosed in the example's module doc, in the new test and in the
  CHANGELOG. So no REQUIRED finding, but see A1.

### (2) Opened deviation
`Opened` holds tree, workspace, tracked paths and symlinks; `Opened::workplan()` calls `Workplan::load_with` on each
call. `DiskTree` has no read cache (checked the type), so each call is a fresh read of the documents; a gate test
(`each_call_of_workplan_reads_the_documents_as_they_are_on_disk_then`) proves it. Within the repository, the example
and `try_check` load only through `open`. D4's intent (one load path; fresh documents for `complete`'s re-check) is
met for documents. Residual: the source `Workspace` and the tracked list are snapshots taken at `open`, so a
re-check after tests have run does not see source or tracked-path changes made meanwhile (A3). `Workplan::load_with`
stays public, so an outside caller can still diverge, which is true of any library.

### (3) git module
* Argv: every function maps to the reconcile Contract row; `every_question_reaches_git_as_exactly_its_argv_in_the_work_directory`
  (recording git over a real process) and the unit tests check them: `symbolic-ref refs/remotes/origin/HEAD`;
  `rev-parse --verify --end-of-options <ref>^{commit}`; `merge-base HEAD <commit>`; `cat-file blob <rev>:<path>`
  as one argument. The ref is a validated `RefName`, later commands receive a `Commit`. A branch named like an option
  is a ref after `--end-of-options` (test) and never reaches git as an option.
* Version: table covers `2.45.1`, `.windows.1`, `(Apple Git-154)`, `2.30`, `-rc1`, `.vfs.0.1`, a second line, and 19
  refusals (overflow, `2.30x`, `-2.30`, wrong prefix, empty). 2.29.9 refused, 2.30.0 accepted, numeric not textual
  order (2.100 > 2.30). Gate/assert_clean do not call the version gate (D4).
* 1 MiB: `Limits::query().with_max_output(MAX_TEXT_BYTES)`; exactly 1 MiB reads (unit and real git), 1 MiB + 1 is
  `BlobTooLarge` and is never cut; a directory is an error, not absent.
* `blob_at` None semantics, probed by hand with git 2.56: an absent path gives "path 'x' does not exist in '<rev>'";
  a path present on disk but not in the commit gives "exists on disk, but not in '<rev>'" (both None, correct, and
  tested including the on-disk case). BUT a nonexistent commit id gives the very same two messages (verified with a
  made-up 40-hex id, with the path absent and with the path present on disk), so a wrong rev is returned as `None`
  ("absent"), not an error. See A2.
* `merge_base`: exit 1 is NoMergeBase, other codes are errors; in a shallow clone that message ("no common
  ancestor") is slightly misleading (A4).

### (4) root::discover
Argv is `Invocation::metadata()` = `cargo metadata --format-version 1 --no-deps --frozen`, program from the runner
(`$CARGO` or `cargo`), 64 MiB bound (`Limits::metadata`; stub test at the bound and bound + 1), one process only.
Scanner: whole text validated (no comments, trailing commas, raw control characters, leading-zero numbers, bad
escapes, BOM, trailing garbage), depth 128 (no stack overflow test), decoded key comparison (so an escaped spelling
of the key is a duplicate), unpaired surrogates in the value refused, nested `workspace_root` ignored, top-level
duplicate refused even with equal values, non-string refused, NUL and relative paths refused, every-prefix and random
punctuation tests never panic. The fixture with `build.rustc`, `rustc-wrapper` and `rustc-workspace-wrapper` scripts
first proves a real `cargo check` runs them (control) and then that `discover` runs none.

### (5) Lookup and preview
`Workplan::change` / `ChangeLookupError` move the old `unknown_change` logic verbatim (same id construction, same
`unloaded` test, same sorted `known`, same findings filter); Display text is shared by `PreviewError`, which converts
variant for variant. HEAD's preview tests pass unchanged, and a new test shows lookup and preview print the same text
for the same slug.

### (6) Mutation re-run (against staged code in a scratch worktree; each reverted)
| Mutant | Result |
| --- | --- |
| version floor 30 -> 29 | killed (3 lib tests) |
| version floor 30 -> 31 | killed (3 lib tests) |
| `--end-of-options` dropped from `git_resolve` | killed (3 lib tests) |
| blob bound -1 / +1 | killed (2 lib tests each, incl. exact-bound test) |
| nested `workspace_root` accepted as fallback | killed (lib json + root tests, tests/root hostile test) |
| tracked fallback widened (`Ok(None)` for any failure, and for any "fatal") | killed (lib + tests/git stub test) |
All baseline runs green first (49 + 18 + 20 + 8).

### (7) Flakiness
git, gate and root test binaries run 10 times through cargo: 30/30 passes. Also ran the gate and git binaries
directly, five rounds, three at a time beside six busy loops: no failure. "text file busy": the cause is the known
race where a forked child briefly inherits the write descriptor of a freshly written stub script. Each test in git.rs
(20 tests, 21 `serial()` mentions), gate.rs (18/19) and root.rs (8/9) holds a process-wide lock from its first write
to its last spawn, and no test thread outside the lock spawns, so within a binary the race is closed; separate test
binaries are separate processes and do not share descriptors. One environment note (A5): the root test
`discovery_does_not_run_the_programs_of_the_cargo_configuration` fails when its binary is run directly on this machine
because the Stage `cargo` shim re-forces a rustc wrapper after the fixture unsets it. Under `cargo test`, where
`$CARGO` is the real cargo, it passes 10/10.

### (8) Hygiene
SPDX on every new file (make reuse 331/331). Scan of added lines for tracker numbers, issue/PR wording, task or
review references, TODO/deferral wording and agent credit or trailers: only the ordinary words "later command",
`ChangeLookupError` and the like. `make issue-refs` test passes. Crate docs, lib.rs overview, Gate/Opened/git/root
module docs and the CHANGELOG entry describe the behaviour built (see A1 and A6 for two inaccuracies).

## Findings

### REQUIRED
None.

### ADVISORY
* A1. Plan text D4 and Task 2 call the example reroute "behaviour-preserving"; it is not (the two exits above, and a
  root with no `.git` entry that sits inside a work tree now lists tracked paths, which affects only link findings;
  the example now also needs `git` on PATH for a root without `.git`). The CHANGELOG discloses the first two
  accurately; the plan sentence should not be quoted in the PR body as is. The generated-docs capability is not
  contradicted.
* A2. `git::blob_at` returns `None` for a commit id that does not exist (git words it as a missing path). The doc
  comment says so and the direction is fail-safe for the intended use (an absent file counts as touched, so an
  obligation is over-reported, never silently dropped), and the only intended rev is the output of `merge_base`. But
  `Commit::new` is public, so a caller can ask a nonsense question and get "absent". Suggest in Task 6: pass only the
  value `merge_base` returned (a `MergeBase` newtype would make this a type fact) and add a test with a made-up id
  that pins the current documented behaviour.
* A3. `Opened` freezes the source `Workspace` and tracked paths at `open`; `complete`'s re-check before the writes
  should call `Gate::open` again, not only `workplan()`, if source or links may change while bound tests run. Say so
  in the `Opened` docs.
* A4. `merge_base` exit 1 reports "no common ancestor"; in a shallow clone the history is truncated, not unrelated.
  Add "or the history is shallow" to the message.
* A5. `tests/root.rs` control build depends on `$CARGO` being the real cargo; behind a shim that re-sets a rustc
  wrapper it fails when the binary is run outside `cargo test` (nextest would run it that way with `$CARGO` set, which
  still fails here). CI uses `cargo test`, so no gate is affected.
* A6. The CHANGELOG says the tracked-path listing keeps "the same behaviour"; it now runs under the runner's limits
  (16 MiB per stream, 300 s, stderr text cut to 2048 characters, wording of failure messages gains "(exit N)"),
  whereas HEAD's inline `git ls-files` had none. A tracked-path listing over 16 MiB (about 200k files) is now an
  error. This matches the Contract-edit bound but is a behaviour change worth one clause in the CHANGELOG.

VERDICT: PASS
