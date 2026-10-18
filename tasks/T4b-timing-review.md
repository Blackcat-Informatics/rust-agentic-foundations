# T4b timing-conversion review (1ef2915..HEAD, ten commits, e5bc866 excluded)

VERDICT: PASS (three non-blocking findings, below)

## (g) Gates run
- `cargo fmt --all --check`: clean.
- `cargo clippy -p raf-source --locked --all-targets -- -D warnings`: clean.
- `cargo test --test repo_gates`: 62 passed, 0 failed (includes `time_in_raf_source_tracked_files_name_no_clock`).
- `cargo build -p raf-source --release --locked`: clean.
- `cargo test -p raf-source --locked`: 0 failed. lib 82 passed; integration binaries sum to 466 passed, 2 ignored (corpus_replay, rustc_oracle; both were ignored before); the two doc-test lines report 10 and 6 passed. About 1m34s wall.
- Repo-wide grep of crates/raf-source/{src,tests} for `std::time|Instant|Duration|sleep|recv_timeout`: no hits.

## (a) Vacuity
Tick sites read: Positions facts.rs:395; ColumnSteps facts.rs:411,416,441,446; Attributes facts.rs:691; PatternSteps glob.rs:94; ExcludeTests members.rs:243; SegmentReads resolve.rs:694,1209,1390; Lookups settle.rs:506,583; ViewReads glob_scope.rs:541,548,562,574; WalkSteps exports.rs:675,1268,1314. All are in code the tests drive, and the tallies are thread-local (work.rs), so work on other threads is not counted. None of the converted tests drives work on another thread.

Every converted counted test has an exact value, or a derived upper bound together with a `> 0` lower bound:
- facts attribute test: `Attributes == 1` (exact).
- facts wide-character line: `Positions >= 2*ITEMS`, `ColumnSteps > 0`, upper bound derived from line length and MARK_EVERY.
- glob segment: `steps > 0` and `<= bound` for each size; the matching case also asserts `> 0`.
- members: `ExcludeTests > 0`, `<= tested*wildcards`, `PatternSteps > 0 && <= steps`; the dedup test is `== 2`.
- resolve: SegmentReads and Lookups, both `> 0` and `<=`, for the missing and the deep path.
- exports `assert_linear`: `one > 0` and `three + one <= 2*two`.
- manifest_caps: Entry `> 0` next to the op equalities; the members-at-cap test compares exact Entry/List/Read deltas.
- module_tree: Open `> 0` and `<=` files+1, OpenedRead `> 0` and `<=`; the inline-alternatives test uses exact counts of 1 and equality with a shallow load.
- disk tests: `recorded_opens` is asserted non-empty (tests.rs:560, and around 1118).
- prescan box pattern: the live-probe control at depth 100 asserts the flag is set.

Tests whose assertion could pass with a zero tally: none that count work.
- Minor (F1): manifest_caps.rs `over.ops() == refused_at_the_manifest().ops()` (in the two cap tests and in the duplicate-probe test) compares two tallies with no `> 0` on either side. If the counting tree never ticked, 0 == 0 would pass. The sibling tests in the same file assert `Entry > 0` on the same `Counting` type, and the probe test also compares `read_log()`, so the risk is small. An `assert!(reference.tree.count(Op::Read) > 0)` in `refused_at_the_manifest` would close it.

## (b) Mutants re-run (scratch clone /tmp/t4b-mut1/c, `git clone --no-hardlinks`; the worktree was not touched; reverted with `git checkout` between runs)
1. facts: `use_role(&group.attrs, ..)` moved into the member loop. `a_large_attribute_on_a_verbatim_use_group_is_read_once_for_all_its_members` FAILED fast: "Attributes work passed the ceiling of 2: 3" (0.18 s).
2. members/manifest: `patterns()` dedup disabled (members.rs:196 `|| true`). `duplicate_patterns_within_the_caps_load_with_the_tree_calls_of_one_of_each` FAILED: "tree operation ceiling 4296 passed by Entry of p" (1.58 s).
3. resolve: `Lookups` tick doubled in `lookup_in` and `lookup_seen`. `a_path_of_fifty_thousand_segments_reads_each_segment_a_bounded_number_of_times` FAILED at resolve.rs:1547, the `Lookups <= 2*segments` assertion (0.68 s).
4. exports: `Surface::bound` reads the views of every name instead of the by-name lookup. `a_module_of_pub_globs_is_listed_in_view_reads_linear_in_its_names` FAILED: "ViewReads work passed the ceiling of 32000: 32001". The WalkSteps test and the carrying-globs ViewReads test still passed, as expected for this mutant. The whole `exports::tests` run took 39 s because the other tests ran in debug builds; the failing test stopped at the ceiling.

All four mutants are killed.

## (c) trait_keys star-heavy test (tests/trait_keys.rs:731-742)
- The guarantee is guarded by `glob::tests::a_segment_is_matched_in_at_most_pattern_times_name_steps` (glob.rs:206-232). It has a PatternSteps ceiling, and per validation.md its recursive-matcher mutant fails at "ceiling of 756: 757".
- F2 (non-blocking): the trait_keys test has no comment saying so. By itself it asserts only that the load succeeds with one package. A backtracking regression would make it hang, not fail, and it would catch nothing the unit test does not. Its name and body claim a behaviour (the member is left alone), which is true, but nothing records that the step bound is held elsewhere. Add a one-line comment pointing at the glob unit test, or fold the test into a plain behaviour test.

## (d) Release behaviour
- Non-cfg(test) changes in disk.rs, linux.rs, handle_path.rs and other_unix.rs:
  - linux.rs: `let flags = O_PATH | O_NOFOLLOW` replaces the inline expression (same value); `note_open` is `#[cfg(test)]`.
  - handle_path.rs: the `open_handle` signature gains a `tree` parameter. The Windows version takes `_tree`, which is unused, so there is no behaviour change. The unix version is compiled only under `all(test, linux)` (disk.rs cfg lines 72/80), so its flags are not part of any release build.
  - other_unix.rs: the only addition is `#[cfg(test)] self.tree.note_open(..)`.
  - disk.rs: `never_blocks`, the `opens` field, `note_open` and `recorded_opens` are all `cfg(all(test, unix))`. The `opens` field and its initialiser are cfg-gated, so the struct layout in release is unchanged.
- The Linux release open flags are unchanged (`O_PATH | O_NOFOLLOW`), and release build and clippy are clean.
- The inotify probe (disk/tests.rs:492-574) is in the test module under `cfg(target_os = "linux")`. It uses `rustix::fs::inotify`, already a dev-dependency, and has negative and positive controls.
- Non-test, non-disk additions: `work::add` compiles to an empty `#[inline]` function outside tests; `mod work;` is private; `tally::add` calls are zero-cost in release.

## (e) Repo gate `time_in_raf_source` (tests/repo_gates.rs)
- Whole-word matching (`has_word`) checks the identifier character on both sides, so `Instantiate`, `sleepless` and `duration-free` pass. The clean-neighbour fixture covers this.
- File reading uses `tracked_paths()`, `read_tracked` and `decode_text`, the same as `issue_refs_tracked_files_are_clean`, and asserts `scanned > 0`.
- Fixtures: a planted violation (file:line per word), each spelling in src and in tests, a clean neighbour, and exemptions (examples/, another crate, a non-.rs file, the `src_notes` prefix). The planted string is assembled so the gate file stays clean.
- `use std::time::*;` is caught by the `std::time` word (the `:` after it is not an identifier character). `use std::time::{Instant}`, `time::Duration` after `use std::time;` and aliased imports are caught by the type words.
- Not caught (F3, non-blocking): `core::time` as a path (only its `Duration` is caught); `tokio::time::{timeout, interval, pause}` and other wait APIs that avoid the listed words (`Condvar::wait_timeout`, `park_timeout`, `set_read_timeout`, `Timeout`); `std :: time` with spaces; `use std::{time, ..}` alone (caught only when a type word follows); `chrono` and other clock crates; a raw string or macro that builds an identifier. It also flags the words inside comments and strings (false positives, which are strict rather than lax). The bypass surface is small: the listed words cover `std` reads. If wanted, add `core::time`, `tokio::time`, `_timeout` and `wait_timeout` to `CLOCK_WORDS`.

## (f) Repo rules in committed text
- A grep of the added lines in crates/ and tests/ for issue/PR numbers, "milestone", "plan", "stage", "task N", "T<N>" and "per the" found nothing; the only matches are the gate's own names. SPDX headers are present on the new files (src/exports/tests.rs, tests/support/counting.rs; work.rs also has them). Names describe shapes (`a_chain_of_files_...`, `..._linear_in_its_names`). The uncommitted changes in docs/ and work/ were ignored as instructed.
- Worktree CLAUDE.md carries the no-timing rule; no timing run, sleep or timing assertion remains in raf-source.

## Findings summary
- F1 minor: manifest_caps.rs `ops()` equality against `refused_at_the_manifest()` has no `> 0` guard on the reference tally.
- F2 minor: tests/trait_keys.rs:731 has no comment recording that the step bound is held by glob.rs:206.
- F3 minor: gate word list misses `core::time`, `tokio::time`, `*_timeout` variants.
