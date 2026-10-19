# T2+T3 review and branch review against the Rev 2 contract (HEAD c28d7e9)

VERDICT: BLOCKED

One finding blocks: B1, which is development-process wording in two committed
files. Everything else in R1 to R6 passes by execution. The non-blocking
findings are N1 to N4.

Method: I used a scratch clone `/tmp/t23rev.V7Z5TB/clone`, made with
`git clone --no-hardlinks` at c28d7e9 (pwd checked before each command). The
scratch registries are `/tmp/t23rev.V7Z5TB/reg` and `/tmp/t23rev.V7Z5TB/reg2`.
Each mutant was reverted with `git checkout --`, and `git status` was clean
afterwards. Nothing was timed: every check is an output, a count or a
pass/fail. The worktree has no edits apart from this file.

## (1) Acceptance grep

* HEAD: `grep -rnE 'std::time|Instant|Duration|sleep|recv_timeout|SystemTime' crates/raf-source/examples`
  printed nothing (rc=1).
* main: `git show origin/main:crates/raf-source/examples/registry_scan.rs | grep -nE ...`
  hits lines 64, 72, 75, 116, 161, 492, 561, 562 and 572. These include the
  plan's baseline of 64, 72, 75, 562 and 572.

## (2) The shipped example over a scratch registry

The registry holds `tiny-1.0.0`, `tiny-0.9.0` and `hub-1.0.0`. The hub is the
`glob_scope.rs` `hub(40)` shape, with 40 globs. `reg2` adds `chain-0.1.0`, a
chain of 8 globs. Release build.

| run | output |
|---|---|
| default | `hub@1.0.0 \| OK ... work=134560` (= 2n^3+4n^2+4n at n=40), `tiny@1.0.0 \| OK ... work=0`, `# largest work=134560 (hub@1.0.0)`; tiny@0.9.0 correctly skipped |
| `--max-work 1000` | `hub@1.0.0 \| ERR Budget :: src/lib.rs:2: exceeds the budget for glob table work steps (1000)`, `# ok=1`, `# err Budget=1`; reg2: `chain@0.1.0 \| OK ... work=16` (0 < 16 <= 1000) |
| `--max-work 1` | hub `ERR Budget :: src/lib.rs:1: ... (1)`, tiny `OK ... work=0` |
| `--max-work 0` | `registry_scan: --max-work must be at least 1`, rc=1 |
| `--max-work` (no value) | usage, rc=1 |
| `--timeout 5` | usage plus ``unknown `--timeout` ``, rc=1 |

**Mutant: the child ignores `--max-work`.** At `registry_scan.rs:379`,
`options.max_work = max_work` was replaced by the default limit. With
`--max-work 1000` the hub then loads `OK ... work=134560`, so the clean run's
refusal shows that the flag reaches the child. **The example's tests do not
catch this mutant** (4 passed); see N1.

## (3) The gate

* `cargo test --locked --test repo_gates`: 68 passed on the tree.
* Real tracked files, planted as comments (the gate matches every line):
  * `crates/raf-source/src/meter.rs:307: std::time` / `Instant`
  * `crates/raf-source/tests/load_budgets.rs:855: std::time` / `Instant`
  * `crates/raf-source/examples/registry_scan.rs:758: std::time` / `Instant`
  * `registry_scan.rs:759: Duration` and `registry_scan.rs:760: elapsed` (these
    two are refused only by `SOURCE_CLOCK_WORDS`)

  `time_in_crates_tracked_files_name_no_clock` failed and listed all eight.
* **Mutant: the old examples carve-out restored in `clock_rule`.** Three tests
  fail: `rejects_a_planted_reading_in_every_ruled_directory`,
  `rejects_each_source_spelling_in_the_library_and_the_tests` and
  `lets_the_runner_name_durations_and_limits`. The last also catches the
  examples being sent to `CRATE_RULE`.
* No exemption is left. `git grep registry_scan tests/` is empty, the
  stale-carve-out assertion and the exemption fixture path are gone
  (`tests/repo_gates.rs:1204`), and `CLOCK_EXEMPT` holds only the clock
  file and the gate file (`tests/repo_gates.rs:326`).
* `grep -n allow crates/raf-source/examples/registry_scan.rs` is empty (rc=1).
* `cargo clippy -p raf-source --all-targets --locked -- -D warnings` is clean.
  `cargo fmt --all --check` is clean.

## (4) The example's unit tests

`crates/raf-source/Cargo.toml:37-39` sets `[[example]] test = true`.
`cargo test --locked --workspace` runs
`Running unittests examples/registry_scan.rs` (4 tests). On the clean tree
the full workspace run reports no `test result: FAILED` suite
(`grep -c` found no match).

**Mutant: `parse_work` accepts 0** (the guard at `registry_scan.rs:143`
changed to `== usize::MAX`). Under `cargo test --workspace`,
`tests::work_limit_refuses_zero_and_names_the_flag` fails at
`registry_scan.rs:697`.

## (5) Builds on Windows and macOS (by reading)

The example has no `std::os::*`, no `ExitStatusExt` and no `cfg` except
`#[cfg(test)]` (`registry_scan.rs:687`). Every function is reached on every
target, so no item is dead on non-unix. `sh` and `/proc/meminfo` are runtime
dependencies only, and `main` had them too. On a host without them the scan
fails with a message (`registry_scan.rs:233`) or prints `ERR spawn` for each
crate. Under the platform policy that is acceptable.

The cross targets are not installed in this toolchain's sysroot, so
`make cross-check` was **not run** here. Its evidence has to come from the
author's run or from CI.

## (6) Blocking wait with no timer

`run_child` (`registry_scan.rs:530-574`):

* stdin is null.
* stdout and stderr each get a drain thread (`:550-551`) before
  `child.wait()` (`:552`), so a full pipe cannot deadlock the wait.
* `exec` replaces `sh`, so no process outlives the child holding the pipes,
  and the joins at `:556-557` return at EOF.
* A child that aborts, overflows its stack or hits `ulimit -v` ends its own
  line (`ERR abort` / `ERR memory`).

In the shipped flow I found no case where the parent hangs while the child has
ended.

**What remains unbounded:** a child that keeps running in uncounted code.

* The documented case is the lookups of settling (`globbed`/`globbed_any`).
  They make a work count of 4n+2 for about 25n^2 glob reads. Only the byte and
  token budgets bound n, so in practice one crate can hang the scan.
* That worker then blocks. The in-order printer (`:662-668`) also holds back
  every later line, and the scan never ends.
* At the default limit (2^31), a hub just under 1024 globs is admitted. That is
  about 900 G instructions, according to the open settling-cost issue.
* The post-load `misplaced` pass is not counted. It is linear in the items.

The module doc (`:31-39`) and the source-index Constraints (`:1720`) state the
lookups limit. The scope has moved to the open settling-cost issue, whose
comment names the lookups shape. So this is not a silent cut.

## (7) Docs

* `CHANGELOG.md:24-30` states that `--timeout` and the `timeout` kind are
  removed, along with `--max-work`, `work=` and `# err Budget`. It is accurate.
* The module doc (`registry_scan.rs:8-67`) is accurate: the usage line,
  `--max-work`, the `work=` field order (it matches `USAGE` and `ok_line`) and
  the outcome kinds without `timeout`.
* `AGENTS.md:162-163`, `CONTRIBUTING.md:95-99` and the gate module doc
  (`tests/repo_gates.rs:22-28`) now say that the examples follow the raf-source
  rule.
* `git grep -i timeout` over the tracked `*.md` files finds no stale
  reference to the example.
* No issue or PR numbers were added (`git diff main..HEAD` added lines,
  checked). B1 is the one process-wording finding.

## (8) Contract R1 to R6

| R | Verdict | Evidence |
|---|---|---|
| R1 counted bound replaces timeout and sleep poll | PASS | (2): `work=134560`; `ERR Budget :: ... glob table work steps (1000)` and `# err Budget=1`; `OK ... work=16 <= 1000` and `work=0`; `--max-work 0` refused; `--max-work 1` gives tiny `work=0`. Task 1 tests pass under `cargo test --workspace`. Limit: N2 and (6) |
| R2 gate extended to `examples/` | PASS | (3): planted readings reported with file:line in `src`, `tests` and `examples`; carve-out mutant caught; clippy clean with no allow |
| R3 acceptance grep empty | PASS | (1) |
| R4 gate passes with examples in scope | PASS | `repo_gates` 68 passed |
| R5 no wall-clock measurement or time-based testing | PASS | the gate passes over all raf-source dirs; no added test reads a clock; the example has no timer |
| R6 planted word in each of src/tests/examples; Duration/elapsed in example | PASS | (3), fixture `SOURCE_RULE_DIRS` (`tests/repo_gates.rs:1014-1018`) used at `:1023`, `:1084` and `:1149`, plus the real-file plant |

**Silently cut:** nothing found.

* The plan's "module doc states `--timeout` removed" is met only through the
  CHANGELOG (N3).
* `make cross-check` was not verifiable here (5).

## Findings

**B1 (blocking): process wording in committed files.**

* `crates/raf-source/examples/registry_scan.rs:38-39` reads "Its fix is the
  settling-cost change tracked in the repository's issue list."
* `docs/capabilities/source-index.md:1720` reads "its fix is the
  settling-cost change tracked in the repository's issue list, which charges
  those reads."

Both were introduced in c28d7e9. CLAUDE.md forbids development-process wording
in any committed file, and a pointer to a tracked change in the issue list is
such wording. Fix: state the limit only, e.g. "The work budget does not bound
this shape." Then re-run `make docs-sync` for the capability text.

**N1: no test catches the `--max-work` plumbing.**

* The mutant that ignores `max_work` in `load_one` (`registry_scan.rs:379`)
  survives `cargo test`.
* A mutant that drops `.arg(max_work.to_string())` at `:540-541` would
  survive too, since only a registry run would show it.
* Suggested test: `load_one` over a temp dir holding a small hub, with
  `max_work = 1`, asserts `ERR Budget`. With the default it asserts `OK`. It
  uses counts only.

**N2:** the default limit (2^31) admits loads of about 900 G instructions on
the hub shape, and the uncounted lookups shape is unbounded in practice; see
(6). Both are documented and moved to the settling-cost issue. They are listed
here so the PR's least-confident note names them.

**N3:** the plan's Design says the `--timeout` removal is "stated in the
module doc and the changelog". The module doc describes only the present flags.
That is arguably better, because a module doc should not narrate history, but
it differs from the plan text.

**N4:** `run_child` returns `ERR wait` at `registry_scan.rs:554` without
joining the drain threads. This is harmless: they end at EOF. It is noted only
for completeness.
