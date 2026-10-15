# T6 review: section-untouched, example-failing, verify

Scope: the staged, uncommitted diff on top of 80c303e (`git diff --cached`): `git.rs` (`MergeBase`,
`default_merge_base`, `blob_at`), `lifecycle/{session,untouched,examples,verification}.rs`,
`obligation` (`RunProblem`, `Needed::PassingRun`), `tests/untouched.rs`, `tests/verify.rs`, support,
CHANGELOG. Experiments ran in a throwaway linked worktree with the staged diff applied (removed
afterwards); the task worktree was not touched.

## Evidence

* Gates (all green on the applied diff): `cargo test -p raf-workplan` (debug and `--release`, doc
  tests included, the two `compile_fail` doctests pass), workspace `clippy --all-targets -D warnings`,
  `RUSTDOCFLAGS=-Dwarnings cargo doc`, `cargo fmt --check`, `docs_contract` (26) and `repo_gates` (57),
  `make reuse` (357/357), `make deny`.
* Flakiness: `tests/untouched.rs` (29) and `tests/verify.rs` (17) passed 5 of 5 runs.
* Mutation: 9 mutants, all killed. No CRLF normalisation (2 tests); no generated-block blanking
  (2); blank range off by one (1); absent merge-base file treated as untouched (1); no laziness
  early return (1); merge-base replaced by the default tip (1); no version gate (4); version gate
  after `Gate::open` (3); `passed()` as `any` (5).
* Hygiene: SPDX on every added file, no tracker numbers, no process wording, no attribution text in
  the diff.
* (2) MergeBase typing: tuple field private, no constructor, `Clone`/`Eq` only; a `Commit` is a type
  error for `blob_at` and `MergeBase(..)` does not construct (both pinned by `compile_fail` doctests).
  `Invocation::git_blob(&Commit, ..)` remains public, but it yields raw output; only `blob_at` maps
  "absent", so the guarded conversion is intact. Fine.
* (4) Version gate: a recording `FakeRunner` shows exactly `["git --version"]` on 2.29, an unreadable
  version also stops after one call, an unstartable git is an error before the repository is read;
  `verify`, `impact` and `new` do not go through it (test asserts verify asks git nothing).
* (5) example-failing: only examples and doc tests resolved in the index are run; test bindings and
  unresolved examples never start cargo (cargo set to a nonexistent program in the test); a failing
  example is owed and the line goes once it builds and runs; a non-example fn is "proves nothing";
  the wrong-role binding also being a `binding-missing` line is acceptable (two distinct true
  statements, tested).
* (6) verify: per-binding lines, no slug = all capabilities, unloaded document / unknown slug /
  unloaded listed capability are errors, hostile bindings are `Unresolved` and never reach cargo,
  tree byte-identical. `passed()` is true for an empty run, which matches the Contract text "exit 0
  means every binding passed"; acceptable, see A2.
* (7) laziness: git is skipped only when every affected section is suppressed by `contract_effect:
  none`; otherwise all questions are asked, so it is not a bypass (mutant 4 pinned).
* (1) digest probes (own experiments): generated-block content blanked on both sides, markers,
  fences, PROPOSED blocks authored; a marker inside a fence is not treated as a generated block;
  heading rename, trailing-whitespace edit, an added blank line, a section moved to the end and a
  removed final newline each count as touched (the digest is exact text, not whitespace-normalised);
  CRLF equals LF; a duplicated heading in the working tree is a profile finding upstream, in the
  merge-base file the first one is compared. Baseline is the merge base (default branch advancing
  after the fork tested), working-tree edits count, file or section absent at the base is touched.
  Errors, not passes, for: no origin/HEAD, no remote, unrelated histories, blob over 1 MiB, non-UTF-8,
  unparsable merge-base file, section missing in the working tree.

## Findings

### REQUIRED

R1. A silent false report when the workspace root is not the top of the git work tree.
Reproduced: a git repository whose top holds `sub/`, the whole workspace (capability documents, change
record, `origin/HEAD`) committed under `sub/`, `Lifecycle::open(Programs::new(<top>/sub))` succeeds
and `reconcile` returns an empty `section-untouched` list although both affected sections are
unchanged since the merge base (the correct list is contract and usage). Cause: `blob_at` is asked for
`docs/capabilities/x.md` relative to the work-tree top, git answers "does not exist in <rev>", which
`blob_at` correctly maps to `None`, and `Files::untouched` then reads that as "new file, touched". The
Contract promises the tool runs from any directory inside the workspace and finds the root from
`cargo metadata`, so a workspace in a subdirectory of a repository (a monorepo) is a plausible setup
and it yields "not reported" where the truth is "owed": exactly the silent pass the Constraints forbid.
The doc stance ("the root is the directory that holds `.git`") is not enough because nothing enforces
it. The linked-worktree case (this repository's `.worktrees/`) is fine: a linked work tree is its own
top, and a test covers it. Fix, either is acceptable: (a) in `Lifecycle::open_with`, after the version
gate, ask `git rev-parse --show-prefix` (or `--show-toplevel`) and return an error when the prefix is
non-empty, or (b) use the prefix to build the `<rev>:<prefix><path>` argument (validated). Both
change the Contract's command list, so add the Contract-edit row and the documents' command table;
add a fixture with the workspace in a subdirectory (error for (a), correct list for (b)) and its
passing neighbour at the top. Preference: (b) if the `SourcePath` rules can carry the prefix, else (a)
with a message naming the toplevel.

### ADVISORY

A1. Whitespace-only edits (trailing spaces, blank lines, a final newline) and moving a section count
as touched. That matches "authored text differs" and is conservative in the wrong direction only for
cosmetic edits (the obligation is cleared by a whitespace change); state it in the `section_digest`
doc, which currently says only "CRLF counts as LF".

A2. `VerifyReport::passed()` and the exit code are 0 for a run with no binding (for example a change
whose capabilities bind no tests or examples). Consistent with the Contract wording, tested and
documented; consider the CLI task printing a note on stderr for an empty run.

A3. `blob_at` recognises "absent" by git's English stderr prefix; this is safe because queries run with
`LC_ALL=C` and everything else is an error, but a one-line comment tying it to the locale guarantee
would protect it from a future runner change.

A4. The merge-base body is parsed but not profile-validated, so a duplicated heading at the base
compares the first section only. No wrong result in a realistic case; mention in the doc if desired.


## Re-review after the fix of R1 (prefix via `git rev-parse --show-prefix`)

R1 is resolved. The implementer runs `show_prefix` once after the version gate; `Prefix::join` puts the
prefix before the document path in the `<rev>:<path>` argument; the prefix must satisfy the
relative-source-path pattern ending in `/`, else `GitError::Answer`.

Evidence (own probe in a throwaway worktree, since removed): the original repro now lists contract and
usage. Prefixes tried, each in a real repository with the whole workspace below the top:
`sub`, `a/b`, `a.b/c-d_e` and a symlink to `real/` and to `a/b` as the opened directory all give the
correct list (git prints the physical prefix); `my dir`, `dossie` with an accent, `q"x`, `a;b` and
`-x` are refused at open with an error that quotes the answer and states the allowed characters, never
a silent result. `--show-prefix` output is not quoted by git, so there is no escaping ambiguity.
Mutant (join dropped, path read at the top) is killed by 3 tests. Version-gate tests still show one
call on 2.29. Gates on the applied diff: `raf-workplan` tests debug and release, untouched/verify 5 of
5 runs, fmt, workspace clippy `-D warnings`, doc `-Dwarnings`, `docs_contract` (26), `repo_gates` (57),
`make reuse`, `make deny` all green. A1/A3/A4 are documented.

New ADVISORY A5: a legitimate directory name with a space or a non-ASCII letter above the workspace is
refused rather than supported. It is a clear error, not a wrong answer, and follows the repository's
path pattern; acceptable.

VERDICT: PASS
