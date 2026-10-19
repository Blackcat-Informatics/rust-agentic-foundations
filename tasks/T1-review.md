# T1 review: commit 1f7a652 (index the coverage product)

VERDICT: PASS

Reviewed by execution in a scratch clone (`git clone --no-hardlinks` to
`/tmp/t1rev.C5r5Qw/repo`, `pwd` checked before each edit, own target dir
`/tmp/t1rev.C5r5Qw/target`). The worktree was not edited. No timing runs; every
check is a count, an equality, or a test pass/fail. The toolchain was not touched.

## 1. Semantics: differential against the old code

Scratch module `coverage/review_diff.rs` (kept at `/tmp/t1rev.C5r5Qw/review_diff.rs`)
holds verbatim copies of the old `matches` and `exclusion_for` from 1f7a652~1 and
compares `ExclusionIndex` with them on the first match (by pointer position) and the
whole matched-flags vector: each exclusion alone, every ordered pair of exclusions,
and the full list forward and reversed, against every entry of one path and every
entry of two paths (so a first match through the second path is covered).

- Well-formed texts (43: depth 1-3 over `a`, `ab`, `b` plus `a::b`, `a::bc`,
  `a::b::c`, `a::b::bc`, `a::b::c::d`): **0 divergences**. Named cases asserted
  equal: Prefix `a::b` against path `a::b` (prefix equal to whole path, no match in
  both); Prefix `a::b` against `a::bc` and `a::bc::x`; Item and Prefix of the same
  text `a::b`; Prefix `a` before and after Item `a::b` (ancestor of an item);
  `[Item x::y, Prefix a]` on entry `["z::q", "a::b"]` and `[Prefix a, Item x::y]` on
  `["a::b", "x::y"]` (first match through the second path).
- Malformed texts without a run of three colons (`""`, `"::"`, `"::a"`, `"a:"`,
  `"a::"`, `"a::b:"`, `"a::b::"`, `"::a::b"`, `"a::b::c::"`, empty path and empty
  exclusion, trailing `::`): **0 divergences**.
- The only divergences: an entry path with an odd run of colons. Prefix `a`
  against `a:::b` (old matches, index does not) and Prefix `a:` against `a::::b`.
  `rsplit_once("::")` on overlapping `::` skips the cut the old `strip_prefix` test
  finds. Not reachable: `exclusion_item`/`path_segments`
  (crates/raf-okf/src/profile/records.rs:51, :448) admit only identifier segments,
  so `a:` cannot be loaded, and Rust paths from the source index cannot hold `:::`
  (an even run such as `m::::x` from a written `::x` agrees). `Bound` uses the same
  `ancestors` rule (crates/raf-workplan/src/coverage.rs:759). Informational, not
  blocking.

## 2. Discriminating tests against the old product code; reviewer mutants

Old product restored in the scratch clone: `ExclusionIndex::exclusion_for` replaced
by the slice walk of 1f7a652~1 ticking a re-added `Work::ExclusionTests`, the
`ExclusionsIndexed` tick removed (the old code built nothing). Result of
`cargo test -p raf-workplan --lib coverage::tests`: 4 failed, 5 passed.

- R2(b) `the_work_grows_with_the_entries_and_the_exclusions_together_and_no_faster`
  FAILS at crates/raf-workplan/src/coverage/tests.rs:294: `4830, 13940, 27450`
  (27450 + 4830 > 2 * 13940).
- R2(c) `every_kind_of_work_is_held_to_a_ceiling_linear_in_the_entries_and_the_exclusions`
  FAILS: `ExclusionTests work passed the ceiling of 4080: 4081`.
- Also failing: the R1 exact-count test (`ExclusionTests ... ceiling of 1408: 1409`)
  and `the_work_grows_with_the_exclusions_and_no_faster` (indexed `[0, 0, 0]`).

Reviewer mutants, none in validation.md (lib `coverage::` and `--test coverage`):

| Mutant | Result |
|---|---|
| MA: the `exclusions.is_empty()` early return removed (coverage.rs:821) | killed: `without_exclusions_the_entries_are_judged_and_nothing_is_looked_up` |
| MB: only the first path of an entry looked up (`paths.iter().take(1)`) | killed: cross product, hand-built duplicates, R1 exact counts; integration `an_item_is_excluded_by_its_exported_or_its_defining_path`, `a_re_export_the_index_cannot_follow_is_reported_and_enforced_and_its_module_covers_it` |
| MC: first found kept instead of the smallest (`first.get_or_insert(position)`) | killed: cross product, R3 check test, hand-built duplicates; integration `the_first_matching_exclusion_gives_the_reason` |
| MD: an uncounted per-entry scan of `self.exclusions` added to `exclusion_for` | survives (9 passed): expected, counts cannot see uncounted work; see 3 |
| ME: `ExclusionMarks` ticked once per non-empty lookup instead of per position | survives: equivalent on every loadable record (see 3) |

## 3. Counts

Ticks sit at the work sites: insert `ExclusionsIndexed` coverage.rs:802, each map
`get` `IndexLookups` coverage.rs:826 and :829, each position marked `ExclusionMarks`
coverage.rs:841; `EntriesJudged` coverage.rs:886. Grep of coverage.rs for iteration
over exclusions: :589 (`check`, exclusions x targets for the needed packages, not
per entry, stated in the capability), :674 (findings, once over the flags), :801
(constructor, counted). `self.exclusions` in the index is used only for
`is_empty()` (:821) and `get(position)` (:833). No per-entry loop over the
exclusions or over a map exists. MD shows such a loop, if added uncounted, would
not be caught by any test; the guard is review, and the code has none. ME survives
because a loaded record never holds two exclusions under one key (duplicate item
texts are refused, records.rs:503; `a::b` and `a::b::*` go into different maps), so
each found list has length one; only the hand-built duplicates test has longer
lists and it does not measure.

## 4. Report unchanged

- Whole suite in the clone: `cargo test -p raf-workplan --locked`: 36 binaries,
  959 passed, 0 failed.
- `cargo workplan coverage code|tests --format json` and `check`: the repository
  record has `exclusions: []`, so its report cannot exercise the index; HEAD and
  the old product give byte-identical `code` and `tests` output and `check` `[]`.
- To exercise exclusions, the scratch record was given nine (Prefix
  `raf_okf::profile::typed::*` matching `raf_okf::Actor` through its defining path,
  a shadowed Item `raf_okf::Actor`, `raf_okf::Actor::ALL`, Prefix and shadowed Item
  under `raf_source::budget::tests`, Prefix `raf_okf::error::*`, unmatched
  `raf_okf::Ghost`, `raf_okf::error::tag_tests`, `raf_okf::Act::*`): HEAD and the
  old product give byte-identical `coverage code` (49049 bytes, 62 excluded),
  `coverage tests` (161722 bytes) and `check` (three
  `coverage-exclusion-unmatched`, 1191 bytes) JSON.
- `crates/raf-workplan/tests/coverage.rs` has no diff in the commit.

## 5. Docs

- Verification text (docs/capabilities/workplan-two-way-coverage.md:258-280) checked
  against the code and tests: index once, lookups per path and module above, equal
  lookups at ten times the exclusions (12 vs 120), growth at fixed entries / fixed
  exclusions / together, ceiling that the product passes (shown in 2), nothing looked
  up with no exclusion, exclusions x targets for packages (coverage.rs:589), the
  cross-product and hand-built claims match tests.rs:421-530. True.
- Bindings: nine new tests bound, four removed, generated table in sync;
  `cargo workplan check` exit 0 with no findings (no `coverage-test-unbound`);
  `assess --list` shows all six assessments of the capability `accepted current`;
  `docs_contract` 28 passed, `self_workplan` 1 passed.
- Added lines grepped for `issue`, `previously`, `no longer`, `replaced`, `was `: 0
  hits. `make issue-refs` exit 0. Commit message has no trailer or number.
- CHANGELOG.md:341 entry under `## [Unreleased]`, accurate.
- `cargo clippy -p raf-workplan --all-targets -- -D warnings` and `cargo fmt --check`
  clean.

## 6. Determinism

`items`/`prefixes` are `HashMap`s used only through `get` (coverage.rs:827, :830);
none is iterated. Each value list is filled in record order and the reason is the
minimum position (coverage.rs:845), so neither depends on hash state; the report
lists are sorted afterwards (coverage.rs:1159-1163). The cross-product test builds
the index twice per list; the differential above also agrees.

## Findings (none blocking)

- coverage.rs:827-831 (info): the index and the old `strip_prefix` test differ on
  entry paths with an odd run of colons (`a:::b`); unreachable from the loader
  grammar and Rust path syntax.
- coverage.rs:842 (minor): `matched.get_mut(position)` skips an out-of-range position
  silently; positions come from `enumerate` over the same slice, so it cannot happen,
  but an indexing panic would state the invariant instead of skipping it.
- tests (info): ME (marks ticked per lookup) is equivalent on loadable records;
  MD (an uncounted scan) cannot be caught by counted tests; neither is present.
