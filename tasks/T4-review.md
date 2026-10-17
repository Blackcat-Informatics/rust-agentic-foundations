# Task 4 review: entity history

## Scope

I reviewed the staged diff on top of de0b842: 16 files, about 3.5k lines. The
staged diff's sha256 is `dcc2858a…` (binary diff with `a/` and `b/` prefixes).
The review tree `/tmp/t4rev` is a detached worktree at HEAD with that diff
applied. Its diff hash matches the staged one byte for byte.

Material read:

* the plan: D11, D12, D18, Task 4, R1 and the edit list;
* the Task 1 to Task 3 reviews;
* the SECURITY.md and CHANGELOG diffs;
* the diff of `docs/capabilities/workplan-entity-history.md`;
* `history.rs`, `history/render.rs`, `git/log.rs`, and the `exec` changes in
  `invocation.rs` and `pattern.rs`;
* `tests/history.rs` and `tests/exec.rs`.

Installed git is 2.56.0. I cross-checked behaviour on git 2.39.5 using the local
`node:24-bookworm` image. I could not install git 2.30 offline: the
debian:bullseye packages would not resolve. I removed that image afterwards.

## Evidence

### 1. Process safety: holds

* **Argv rows.** All five argv rows match the Contract code block exactly,
  including where the options sit and `--since` before the last `--`.
  * The pinned tests `every_invocation_is_the_row_of_its_table`,
    `every_question_is_asked_in_order_as_exactly_its_argv` and the stub argv
    test prove this.
  * The stub records each argv element separately.
* **The id never reaches git.** It is only matched against loaded headers.
  * `coverage:` gives UnknownKind.
  * `cap:-rf` and `evidence:x` give Unknown.
  * A file that exists but did not load gives NotLoaded with its findings.
  * In every case no process starts (tested).
* **TreePath:**
  * accepts `-rf.rs`, `--output=x.rs`, spaces, unicode and inner `:`;
  * refuses a leading `:` or `/`, `*?[\`, control characters, empty, `.` and
    `..` segments, and anything over 4096 bytes.
  * So pathspec magic (`:(top)`, `:!`) and globs cannot be built.
  * Against real git:
    * `-L1,1:a:b.rs` reads the file `a:b.rs` (the path after the range's colon
      is taken literally);
    * `-- sub/-rf.md` is a path;
    * a name like `:(top)z.md` is refused by the pattern.
* **Identifier** is `[A-Za-z_][A-Za-z0-9_]*`.
  * `last_segment` strips a `<...>` argument list and an `r#` prefix, so
    `r#type` becomes `type` and a keyword is harmless inside `-S`.
  * A generic like `<T::X>` gives a refusal before any process starts, not an
    injection.
* **Crate directories** come from `cargo metadata` package dirs, matched by
  the target's crate name.
  * The root package is `.` (TreePath::top).
  * Nested packages are included under their parent's directory (advisory 5).
* **`<commit>^`** is built only from a `Commit` (40 or 64 hex digits) that git
  printed and the pattern accepted. The parent's answer is validated too.
  * A file named `<sha>^` in the work tree makes `rev-parse` echo the argument.
    That gives `GitError::Answer`, not a silent `None`.
* **Hostile output** (fake and stub runners):
  * a subject with 0x1e or 0x1f, ESC, `|` or a hash-like text is kept whole;
  * a CRLF subject is fine (git itself strips the trailing CR from `%s`);
  * an invalid UTF-8 subject becomes U+FFFD;
  * a non-UTF-8 path, a malformed quoted path, a bad date or a bad commit is a
    hard error;
  * quoted paths are unquoted correctly (checked against real
    `core.quotePath` output for `ü é"q.md`);
  * a patch line under `--no-patch` is refused.
* **16 MiB bound:** exactly the bound is read and bound + 1 is
  `LogTooLarge`, with both a fake runner and a real 17 MB stub process.

### 2. Semantics against real git (scratch fixtures and plain git)

* **`--follow --name-only`:**
  * a pure rename and a rename with edits are both followed;
  * the rename commit lists the new name and earlier commits list the old
    name, which matches the Contract;
  * a document deleted and re-added keeps its full chain.
* **Clean merges** are not listed. That is consistent, because a clean merge
  changes nothing.
* **A conflict-resolution merge** that changed the document is NOT listed by
  `--follow`, though plain `git log -- path` lists it. This is REQUIRED 2.
* **`-L`:**
  * A function changed in 1 of N commits that touch its file lists only that
    commit plus the commit that wrote it.
  * On 2.56, `-L` follows both a pure rename and a rename with edits.
  * A function moved to another file starts at the move.
  * A conflict-resolution merge is listed.
  * The Constraints text ("follows a rename Git detects, ends where the lines
    came from another file") is accurate. It correctly replaces the plan's
    earlier wording that the log "ends where the file is renamed".
* **Pickaxe (`-S <id> -n 1 -- crates/a`):**
  * multiple removals report the last one;
  * deleting the whole file is reported;
  * a later comment that mentions the identifier is reported instead (stated
    limit);
  * renaming to a longer name (`renamed` to `renamed_v2`) keeps the count, so
    the report names the ADDING root commit and "last resolved: none" (stated
    limit, but see advisory 3);
  * an identifier that never existed gives NoCommit;
  * a nested crate's occurrence is reported (advisory 5).
* **Ordering:**
  * entries are sorted by author instant, then by commit id;
  * equal instants in different zones are ordered by id (fake-runner test);
  * the sort is deterministic.
* **Shallow clones and worktrees:**
  * `--depth 1` is refused;
  * a linked worktree of a shallow clone answers `true`, including from a
    subdirectory;
  * after `fetch --unshallow` it answers `false`.
* **Grafts and replace refs** are honoured the same way by `log` and
  `rev-parse`. That is trusted repository state.
* **Real use** (scratch test through `Gate::open` on this repository, in a
  linked worktree):
  * `cap:workplan-reconcile` gives 14 entries, with document and symbol
    sources;
  * `arch:normal-form` gives 2 entries.

### 3. `--since` depends on the clock

Git reads a bare `YYYY-MM-DD` as that day at the current local time of day.

* At 22:35 MDT, `git rev-parse --since=2026-05-01` gave
  `--max-age=1777696521` (2026-05-02T04:35:21Z).
* In a three-commit fixture dated 2026-05-01 at 00:30Z, 12:00Z and 23:30Z,
  `--since=2026-05-01` listed none of them.
* `TZ=Asia/Tokyo` gave yet another cut-off.

The Contract even says so ("Git reads it as that day at the current time of
day"). So the documented behaviour drops or keeps commits depending on the
clock and the time zone. Per the brief, this is REQUIRED 1.

The fixed forms are deterministic and ignore the time zone. Each of these gave
`--max-age=1777593600` on both 2.56.0 and 2.39.5:

* `2026-05-01 00:00:00 +0000`
* `2026-05-01T00:00:00Z`
* `2026-05-01T00:00:00+00:00`

### 4. Linked worktrees

Covered under section 2. Git older than 2.15 is moot, because 2.30 is the
minimum and is refused below that (2.29 stub test).

### 5. Determinism, rendering and escaping

* Table and JSON carry the same entries in the same order (tested).
* JSON escapes `"`, `\` and every C0 character. DEL and U+2028 pass through
  raw, which is valid RFC 8259.
* Table cells use the generated-table escape, and control characters become
  `\u{..}`.
* Unicode bidi or format characters (U+202E and similar) are not escaped
  (advisory 6).

### 6. Lookup errors

* Unknown, not-loaded and wrong-kind ids each produce their own error.
* An unknown id's error lists the loaded ids of its kind.
* `evidence:` and `assessment:` ids resolve to `work/evidence/` and
  `work/assessments/` (tested).

### 7. Documents

* **`workplan-entity-history.md`:** a scripted line diff shows zero changed
  lines outside PROPOSED blocks. The state is still `considered` and all
  bindings are empty.
* **SECURITY.md:** its statements are true of the code, apart from the
  `--since` form after the fix.
* **CHANGELOG:** accurate.

### 8. Mutation re-run (`--lib`, `--test history`, `--test exec`)

All six mutants were killed:

| Mutant | Killed by |
| --- | --- |
| `--` before the pathspec dropped | 8 tests |
| `--since` placed after `--` | 6 tests |
| shallow check removed | 5 tests |
| TreePath accepting a leading `:` | pattern tests |
| dedupe keyed per source | 5 tests |
| parent answer not validated (`.ok()` mapping to `None`) | `the_parent_is_a_commit_none_for_a_root_and_an_error_otherwise` |

### 9. Hygiene

* SPDX headers are present on every new file.
* The added lines contain no tracker or process wording, no attribution and no
  `unsafe`.
* No manifest or lockfile changes, so no new dependency; the JSON is escaped
  by hand.

### 10. Gates (on `/tmp/t4rev`, `CARGO_BUILD_JOBS=6`)

* **raf-workplan, release:** rc 0, 769 passed.
* **raf-workplan, debug:**
  * The first run had one failure:
    `sync::the_docs_sync_example_opens_the_repository_as_the_gate_does`
    exited 101 where 2 was expected. That is the known flake from the Task 3
    advisory.
  * `--test sync` then passed 3 out of 3 times.
* **`cargo test --workspace --locked --no-fail-fast`:**
  * The first run returned rc 101, with the failing test's output not kept.
  * The re-run returned rc 0 with 1747 passed and 0 failed, including
    `docs_contract` (28) and `repo_gates` (57).
* **`make fmt`, `clippy`, `doc`, `deny`, `reuse`, `msrv`, `issue-refs`:**
  rc 0.
* **`make docs-sync` twice:** rc 0, no change to the tree either time
  (idempotent).

## Findings

### REQUIRED

1. **`--since` is clock- and time-zone-dependent.**
   * **Fix:** keep the `Date` pattern for the user value, but have `Arg::Since`
     write one argv element `--since=<date> 00:00:00 +0000`.
     * That is git's own `--date=iso` shape, read by `parse_date_basic` with a
       numeric offset in every git from 2.30 on.
     * Both it and `<date>T00:00:00Z` were verified on 2.39.5 and 2.56.0.
     * Prefer the numeric offset: an older git may not know the letter `Z`.
   * **Text to update:**
     * the Contract code block and the sentence "Git reads it as that day at
       the current time of day", which should become "from 00:00:00 UTC of
       that day";
     * SECURITY.md (`--since=<YYYY-MM-DD>`);
     * the CHANGELOG;
     * the `Pattern::Date` and `Arg::Since` docs;
     * the pinned rows in `tests/exec.rs` and the stub and fake argv in
       `tests/history.rs`.
   * **New real-git fixture:** commits at 23:30Z the day before, and 00:30Z
     and 23:30Z on the `--since` day.
     * Assert the exact set: the day before is excluded, both same-day commits
       are included.
     * Run it under at least two `TZ` values, for example `Pacific/Kiritimati`
       and `Etc/GMT+12`.
2. **The document log silently omits conflict-resolution merges.**
   * `git log --follow --name-only -- <doc>` leaves out a merge commit whose
     resolution changed the document. The fixture's "conflict merge" commit is
     missing, so the report omits a commit that shaped the document's current
     text.
   * **Fix:** add `--cc` to the document row.
     * Verified on 2.56 and 2.39: it lists the merge with its path, a clean
       merge is still omitted, and the record parser already handles it.
     * Update the Contract row, the SECURITY text and the pinned rows.
     * Add a real-git merge-conflict fixture.
   * **Alternative:** state the omission in Constraints. That is weaker,
     because the result would be knowingly incomplete.
3. **`log.showRoot=false` silently drops the root commit.**
   * With this user config, the document log loses the commit that created the
     document.
   * The pickaxe returns nothing for a symbol added in the root commit, which
     is reported as `NoCommit`.
   * The design already neutralises `log.showSignature` for the same reason.
   * **Fix:** add `--root` to the document and pickaxe rows.
     * Verified: with `log.showRoot=false` set, `--root` restores the root
       commit on 2.56 and 2.39.
     * The `-L` row is unaffected.
   * **Fixture:** `git config log.showRoot false` in a fixture repo.

### ADVISORY

1. **Paths in an entry mix two bases when the workspace is below the top of
   the work tree.**
   * Document paths come from `--name-only` and are relative to the top
     (`ws/docs/b.md`).
   * Symbol files and `History::path()` are relative to the workspace root.
   * `diff.relative=true` flips the document paths.
   * Normalise them (strip the `--show-prefix` the gate already reads), or
     state this in the Contract.
   * Also, "the paths it touched" is only the document's own name and the
     bound files, not every path in the commit (`--full-diff` cannot be
     combined with `--follow`). The Contract wording should say so.
2. **`-L` fails hard when a bound file is uncommitted or shorter in `HEAD`.**
   * It gives `GitError::Failed`, which is common while a capability is
     `implementing`.
   * This is fine as fail-hard, but the Contract or Constraints should say it.
3. **State the pickaxe limit more sharply.** When the count is unchanged (a
   rename to a longer name, or a module move), the reported "remover" can be
   the commit that ADDED the symbol, and "last resolved" a commit in which it
   never existed. Options:
   * say exactly that;
   * or add a direction check (does the identifier occur in `<commit>^` but
     not in `<commit>` under the directories?);
   * and/or print `%P` in the pickaxe format, which drops both the separate
     `rev-parse` and the matching of git's English stderr.
   * Also note that removals made inside merges are not seen.
4. **`git rev-parse <commit>^` without `--verify`** echoes the argument when a
   same-named file exists. It fails safe (`Answer`), but `%P` (advisory 3)
   removes the case entirely.
5. **Crate directories include nested packages.** A nested package's
   occurrences count toward its parent crate.
   * For the root package `.`, the search covers the whole repository: docs,
     CHANGELOG, and the capability document itself.
   * The stated limit covers this only loosely. Say "the crate's directory,
     including any nested package and, for a root package, the whole
     repository".
6. **Table cells do not escape Unicode bidi or format characters** (U+202A to
   U+202E, U+2066 to U+2069), so a subject can reorder how a terminal shows
   it. Consider escaping the `Cf` category too.
7. **A partial (blobless) clone triggers lazy fetches** during `-L` and `-S`.
   That is network traffic and writes into `.git`, which contradicts "reads
   Git and writes nothing". Consider `GIT_NO_LAZY_FETCH=1` (git 2.44 and
   later) or stating the limit.
8. **Verification bullets in the entity-history document:**
   * "the report lists that commit only" should read "that commit and the one
     that wrote it", to match the test;
   * "stub `git` on `PATH`" is actually `Programs::with_git`; recheck this when
     the CLI lands in Task 9.
9. **Gate flakes.** The `sync` example test flaked once (known, from Task 3).
   One workspace run also returned 101 once, with its output not kept; the
   re-run was clean.

Round one verdict: BLOCKED (superseded by round two below).

## Round two

### Scope

I re-reviewed the fixes for REQUIRED 1 to 3 and the advisories.

* Staged diff sha256: `f70c353d…` (16 files, +3713/-108).
* I applied it to a fresh detached worktree at HEAD, `/tmp/t4rev`.
* The worktree's diff hash matched before and after the mutant runs.

### Evidence

* **REQUIRED 1 (`--since`):** fixed.
  * **The argument:** `Arg::Since` now writes one argument,
    `--since=<date> 00:00:00 +0000`. SECURITY.md, the Contract code block
    (quoted as one argument), the CHANGELOG and the `Pattern::Date` and
    `Arg::Since` docs all say "from 00:00:00 UTC".
  * **Real git:** my own fixture has a root commit at 2026-04-30T23:30Z and
    nine commits on 2026-05-01 from 00:30Z to 23:30Z. The new form gave the
    same set under `UTC`, `Pacific/Kiritimati`, `Etc/GMT+12` and
    `America/Denver`: the root was excluded and every commit of that day was
    included.
  * **The implementer's fixture**
    (`since_counts_from_the_start_of_the_day_in_utc_in_every_time_zone`)
    runs git through a per-zone wrapper.
    * Under the bare form, `Etc/GMT+12` always moves the cut-off past 00:30Z,
      so the fixture kills the bare-date mutant at any time of day.
* **REQUIRED 2 (`--cc`):** fixed.
  * **My fixture** has one merge of each kind:
    * a clean no-ff merge;
    * a conflict-resolution merge;
    * an `-s ours` merge that discards a side edit;
    * an "evil" merge (clean, plus an extra edit to the document);
    * a rename at the end.
  * **What the new row lists:**
    * every non-merge commit that touched the document;
    * the conflict-resolution merge and the evil merge, each with its path.
  * **What it does not list:** the clean merge and the `-s ours` merge. Those
    are the only path-less records, and they are dropped.
  * **Dropping path-less records is safe.**
    * Only merges can be path-less here. A non-merge commit that touches the
      pathspec always prints its name under `--name-only`, and so does a root
      commit under `--root`.
    * Under `--cc`, a merge is path-less exactly when its result equals one
      parent.
    * So the drop never removes a commit that changed the document relative
      to every parent.
    * The `-s ours` merge, which reverted a side edit, is omitted. Plain
      `git log -- <path>` omits it too, under default history simplification.
      This matches the Contract sentence closely enough (advisory R2-1).
  * **The implementer's fixture**
    (`a_merge_whose_resolution_changed_the_document_is_in_its_history_and_a_clean_one_is_not`)
    covers the conflict merge and the clean no-ff merge.
* **REQUIRED 3 (`--root`):** fixed.
  * `--root` is on the document row and the `-S` row. `-L` does not need it,
    because it lists the root without it.
  * With `log.showRoot=false`:
    * the document log lists the root commit (real git, mine and theirs);
    * the pickaxe finds a symbol added in the root commit with an empty `%P`;
    * the test repository sets `log.showRoot false` for the fixtures.
* **`%P` change:** correct.
  * `rev-parse`, `Arg::Parent` and `git::parent` are gone, along with the
    matching of git's English stderr.
  * The format is `%H%x1f%P%x1f%aI%x1f%s`.
    * It splits into three parts (commit, parents, rest).
    * The rest is rejoined with the commit and parsed by the same `header`,
      so the subject keeps its 0x1f bytes.
  * Parents are validated:
    * an empty `%P` is the root, so no parent;
    * otherwise each space-separated id must pass the commit-id pattern, and
      any bad id is `GitError::Answer`;
    * the first id is the first parent.
  * Two mutants confirm this (M6 and M13 below).
  * Real git gives an empty `%P` for a root commit. `-S` does not diff
    merges, so a merge never appears as the pickaxe answer. That is now
    stated: "A removal inside a merge is not seen."
* **Advisories:**
  * **Bidi escaping:** U+061C, U+200E/F, U+202A–E and U+2066–9 are escaped in
    table cells and code spans, and other non-ASCII text is kept (tested).
  * **Pickaxe limits:** the Constraints text now states nested packages, the
    whole repository for a root package, that an adding commit can be
    reported, and that merges are not seen.
  * **Uncommitted or shorter bound files:** the hard failure is stated.
  * **Verification wording:** fixed.
  * **"Paths it touched":** narrowed to "the paths of the document and of the
    bound files it touched".
* **Documents:**
  * The entity-history document is still `considered` with empty bindings,
    and every edit is inside a PROPOSED block (re-checked).
  * SECURITY.md and the CHANGELOG match the code.

### Mutants

All 13 were killed (`--lib`, `--test history`, `--test exec`):

| Mutant | Result |
| --- | --- |
| M1: `--` dropped | killed |
| M2: `--since` after `--` | killed |
| M3: shallow check removed | killed |
| M4: TreePath accepting `:` | killed |
| M5: dedupe keyed per source | killed |
| M6: a bad parent id silently filtered out | killed by `the_pickaxe_answers_one_record_with_its_parent_or_none` |
| M7: bare `--since=<date>` | killed, including by the TZ fixture |
| M8: no `--cc` | killed, including by the merge fixture |
| M9: no `--root` on the document row | killed, including by real-git fixtures |
| M10: no `--root` on the pickaxe | killed by the root-commit fixture |
| M11: path-less records kept | killed by the merge fixture and a unit test |
| M12: bidi left unescaped | killed by the render test |
| M13: last parent instead of first | killed |

### Gates (`CARGO_BUILD_JOBS=6`)

* `make fmt`, `clippy` and `doc`: rc 0.
* `cargo test --workspace --locked --no-fail-fast`: rc 0, 1748 passed,
  0 failed. This includes `docs_contract` and `repo_gates`, and no flake
  recurred.
* `cargo test -p raf-workplan --release`: rc 0, 770 passed.
* `make deny`, `reuse`, `msrv` and `issue-refs`: rc 0.
* `make docs-sync` twice: rc 0 both times, and the tree's status was
  unchanged after each run.

### Hygiene

* SPDX headers are present.
* No tracker or process wording, attribution or `unsafe` in the added lines.
* No manifest or lockfile changes.

### REQUIRED (round two)

None.

### ADVISORY (round two)

1. **The Contract sentence about skipped merges** reads "a merge that changed
   nothing in it is not an entry". A merge whose result equals one parent but
   discards the other parent's edit (`-s ours`, or a resolution that keeps
   one side) is also not an entry, though it did change the document
   relative to that parent. "A merge whose result equals one of its parents"
   would be exact.
2. **Mixed path bases are still open.** Below the top of the work tree,
   document paths are relative to the top while symbol files are relative to
   the workspace. This is stated only in the rustdoc, not the Contract.
   Carried from round one, advisory 1.
3. **Partial clones** (round one, advisory 7) still fetch objects lazily. The
   limit is not stated.
4. **The `sync` example flake** (Task 3) did not recur this round.

VERDICT: PASS
