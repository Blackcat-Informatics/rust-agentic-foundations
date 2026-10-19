# Gap analysis: PR 31 / issue 27 (Stage 2, lightweight combined)

* **Branch:** paudley/27-raf-source-the-registry-scan-example
* **HEAD:** 797d600
* **Base:** main
* **Changes:** 6 commits, 17 files, +822/-199

**Method:**
* Read the issue (body plus its 1 comment, the Rev 2 plan, which is identical to
  plan.md) and the PR (body identical to pr-body.md; 1 comment, the plan; 0 inline
  comments; 0 reviews), in full.
* Read issue 28 (its comment names the lookups shape).
* Ran the checks in a scratch clone, `/tmp/gap27.oYR3UR/repo`, made with
  `git clone --no-hardlinks` at 797d600.
* Ran two plumbing mutants there and reverted them; `git status` was clean afterwards.
* The worktree was not edited, apart from this file.
* No timing runs: every figure is a count, a pass/fail result or an exit code.

## Verdict: PASS. No CRITICAL, HIGH or MEDIUM gaps; 6 LOW.

## Issue acceptance criteria

| Criterion | Status | Evidence |
|---|---|---|
| `grep -nE 'std::time\|Instant\|Duration\|sleep\|recv_timeout\|SystemTime' crates/raf-source/examples` is empty | MET | `grep -rnE` (recursive) rc=1, and the literal non-recursive form rc=1. main has 9 hits. |
| The repository gate passes with `examples/` in scope | MET | `cargo test --locked --test repo_gates`: 68 passed. `clock_rule` sends raf-source `examples` to `SOURCE_RULE`, and the carve-out arm and its stale-carve-out assertion are gone. Hosted `test` passed. |

## Plan contract R1-R6

| R | Status | Evidence on HEAD |
|---|---|---|
| R1 counted bound replaces the timeout and the sleep poll | MET | See (a) to (c) below. |
| R2 gate extended to `examples/` | MET | `SOURCE_RULE_DIRS` has 3 entries, used by the planted-word, spelling and Duration/elapsed fixtures. `examples/a.rs` was added to the clean-neighbour fixture, and the exemption fixture path was removed. clippy `-D warnings` is clean, and `grep -rn allow crates/raf-source/examples` rc=1. |
| R3 acceptance grep empty | MET | as above |
| R4 gate passes with examples in scope | MET | repo_gates 68/68; `make issue-refs` exit 0 |
| R5 no wall-clock measurement or time-based testing | MET | The gate covers the example. The new example test is count-based: `load_one` at the default limit gives OK, and at 1000 gives `ERR Budget`. |
| R6 planted words in src, tests and examples; Duration and elapsed in examples | MET | `time_in_crates_lets_the_runner_name_durations_and_limits` loops over `SOURCE_RULE_DIRS`. T23 plant results are confirmed by the HEAD tests passing. |

R1 evidence:
* (a) The Task 1 tests pass: lib 93, load_budgets 23.
* (b) The example has a blocking `Child::wait`, `--max-work`, `work=` and
  `ERR Budget`. The T23 shipped run gave `work=134560` at n=40, and
  2*40^3 + 4*40^2 + 4*40 = 134560.
* (c) The new `the_work_limit_reaches_the_load` test kills the mutant
  `options.max_work = default` (`registry_scan.rs:800`, panicked).

## Earlier review findings on HEAD

| Finding | Status | Evidence |
|---|---|---|
| T1 F1 (lookups uncharged) | CLOSED (disposed) | Stated as fact in the example module doc (`registry_scan.rs:35-38`) and in the source-index Constraints (`:1720`), with the shape and figures. The scope moved to issue 28, whose comment names the shape. |
| T1 F2 (32-bit claim on `WORK`) | CLOSED | The `budget.rs:128-129` comment now reads "2^31 steps: the same on every target". Residue: see L4. |
| T1 F3 (search refusal names the first glob) | CLOSED | `source-index.md:767-769` says the search steps are charged to the first glob import of the load. |
| T1 F4 (CHANGELOG omits `read_glob`) | CLOSED | The CHANGELOG now lists "each name a new reader is given". Residue in the rustdoc: see L2. |
| T23 B1 (process wording) | CLOSED | Both sentences were replaced in 797d600. In the added lines of `git diff main...HEAD`, `grep -iE 'issue\|tracked\|\bPR\b\|pull request\|plan\|task\|#[0-9]+'` finds nothing, and so do follow-up/todo/defer/review/stage/commit. `make issue-refs` exit 0. |
| T23 N1 (`--max-work` plumbing untested) | PARTIAL | The `load_one` side is covered and its mutant is killed. The `run_child` side is not: see L1. |
| T23 N3 (module doc on the `--timeout` removal) | CLOSED | `registry_scan.rs:40-42` says there is no `--timeout` and no `timeout` outcome. |

## PR body claims against evidence

| Claim | Status | Evidence |
|---|---|---|
| Max work 34343004 (`web-sys`) over 1271 crates; "a sixty-second" of the default | VERIFIED | See the notes below the table. |
| Chain 2n, fan-in 4n^2+6n, prelude n(15n+17)/2, hub 2n^3+4n^2+4n | VERIFIED | The tests assert them at n = 8, 16 and 32 (pass), and T1's probe confirms them at 64 and 128. |
| `work=134560` at n=40 | VERIFIED | It equals the formula, and appears in the T23 shipped run. |
| The hub passes the default at 1024 | VERIFIED | n=1023 gives 2,145,388,542, below 2^31. At n=1024, 2n^3 alone equals 2^31. |
| "about 9e11 instructions" at the default | PLAUSIBLE | See L6. |
| Lookups shape 4n+2 versus about 25n^2 | VERIFIED | T1 probe; it is in the capability and the example doc, and the issue 28 comment exists. |
| Local gates exit 0 | VERIFIED (subset rerun) | Rerun here: check (workspace and fuzz), clippy (workspace and fuzz), fmt, doc `-D warnings`, test -p raf-source, repo_gates, docs_contract, issue-refs. |
| cross-check covered by hosted CI | VERIFIED | Run 39197636507 at headSha 797d600: all 12 checks pass, including cross-check, test-windows and test-macos. Mergeable, CLEAN. |

Notes on the 34343004 claim:
* `/tmp/f27-scan.txt` (20:38) shows `# crates=1271`, `# ok=1251` and
  `# largest work=34343004 (web-sys@0.3.106)`.
* The registry index-...1949cf8c has 1271 distinct crate names.
* No `src` change since 1531b15 except a comment, so the run stands for HEAD.
* 2^31 / 34343004 = 62.5.

## Consumers

* `cargo check --workspace --all-targets --locked`: exit 0.
* Fuzz workspace: `cargo check --all-targets --locked` and clippy `-D warnings`, exit 0.
* docs_contract: 28 passed, so the bindings and digests resolve.
* The example's unit tests run under the workspace (`test = true`): 5 passed.
* README, AGENTS.md (`:151-163`), CONTRIBUTING.md (`:89-99`) and the
  `tests/repo_gates.rs` module doc (`:19-29`) agree: src, tests and examples
  are in scope, and raf-source examples are under the stricter rule.
* `git grep` finds no stale "outside the gate", "clippy only", "carve-out" or
  `--timeout` text.
* `.deficiencies`: the notice and marker only, no entries, untouched by the branch.

## Gaps

### CRITICAL
None.

### HIGH
None.

### MEDIUM
None.

### LOW

* **L1 (`run_child` work limit untested):** the work limit that `run_child`
  passes to the child is not checked by any test.
  * Mutant tried: `registry_scan.rs:544`, `.arg(max_work.to_string())` replaced by
    the default limit, with `max_work` consumed. It survives: 5/5 pass.
  * The only evidence for this side is the manual scratch-registry run in T23.
  * Possible fix: a test that runs `run_child` with
    `std::env::current_exe()`. That needs `--one` handling in a test binary, so it
    may not be worth it. Otherwise accept the run evidence.
* **L2 (`Budget::Work` rustdoc incomplete):** the rustdoc (`budget.rs:68-75`)
  leaves out "each name a new reader is given" (`read_glob`). The CHANGELOG and
  `source-index.md:760` list it, so the three descriptions of what is charged
  disagree. This is the residue of F4.
* **L3 (stale registry count):** `source-index.md:701` still quotes "1177 crates
  of a local registry" for the `max_total_bytes` default, while `:774` quotes the
  1271-crate run. The 1271 run gives the same largest load (`linux-raw-sys`, 16893229
  bytes), so the derived default is unchanged; only the population count is out of
  date. The tokens (1505567) and recursion (987) figures at `:654` and `:672` also
  match the 1271 run.
* **L4 (redundant const assert):** `budget.rs` tests keep
  `const { assert!(WORK < usize::MAX) }`. Its rationale comment was removed under
  F2, and it is trivially true on every supported target. It can go, or stay as
  harmless.
* **L5 (thin `WORK` comment):** the plan's Design says the default "is set above
  the highest figure ... with the figure and the shape that reached it documented
  next to the limit, as for the byte budgets". The figure (`web-sys`, 34343004) is in
  the capability's `max_work` entry, as the `TotalBytes` figure is. The `WORK` const
  comment (`budget.rs:128`) gives no rationale, unlike `FILE_TOKENS`.
* **L6 (instruction estimate source):** the PR body's "about 9e11 instructions"
  comes from issue 28's audit (890.6 G at 1,000). T1's per-step perf extrapolation
  (about 570 instructions per step) gives about 1.2e12. Both are about 1e12, and the
  body says "about". The capability states the 1024-sibling bound, but not an
  instruction figure at the limit.

### Moved scope (not a gap of this change)
* The uncounted lookups of settling (`globbed`/`globbed_any`).
* The cost of the hub at the default limit.

Both are stated as facts in the capability Constraints and the example doc, and are
tracked by issue 28 (open, with a comment naming the shape).

## Recommended actions (all optional; none blocks merge)

1. L2: add "each name a new reader is given" to the `Budget::Work` rustdoc, then
   `make docs-sync` if the digests move.
2. L3: change "1177" to the 1271-crate population at `source-index.md:701`, or
   say which run each figure comes from.
3. L1: accept the T23 run evidence, or add a child-spawn test.
4. L4 and L5: cosmetic.
