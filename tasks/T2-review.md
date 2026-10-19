# T2 review (commits 2e51549, defd0e2, 409779d, 3f0a34b)

VERDICT: BLOCKED (first pass at 3f0a34b; superseded by the delta recheck below, final VERDICT: PASS)

Gates, run in a fresh `git clone --no-hardlinks` at 3f0a34b (/tmp/t2rev.flmpoM/repo):
`cargo fmt --all --check` clean; `cargo clippy -p raf-workplan --locked --all-targets -- -D warnings` clean;
`cargo test -p raf-workplan --locked --lib -- work:: graph:: rationale:: assessment:: obligation::` 52 passed.
(Filters must follow `--`; the form in the brief errors on the second filter.)

## Blocking finding: Task 1's flaw is repeated; three regression mutants pass

Each mutant was applied in the scratch clone, tested with the module filter, and reverted.

| Mutant | Edit | Result |
|---|---|---|
| M1 | crates/raf-workplan/src/obligation/sources.rs:200-225: `known` is a `Vec<&str>`, checked by `known.iter().any(..)` per item (quadratic: items x records) | PASS (not caught), 23 obligation tests ok |
| M2 | crates/raf-workplan/src/rationale.rs:181: `partition_point` replaced by `iter().position(..)` scan from the start of the prose lines | PASS (not caught), 5 rationale tests ok (run took 8.8 s, so the scan really is slower) |
| M3 | crates/raf-workplan/src/graph.rs:1345-1368: `loaded` is a `Vec<&str>`, checked by `iter().any(..)` per claim slug | PASS (not caught), 4 graph tests ok |

## Per counter: where it ticks, and a mutant it cannot see

- BlankedSteps, graph.rs:1826: ticks per `ranges.next()`, at the work. Not seen: the `sort_unstable`/merge pass (graph.rs:1800-1815), and replacing the peekable cursor by a scan of `merged` per character (ProseChars is unchanged and BlankedSteps only drops, which an upper bound `<= units*UNIT` allows; only `one > 0` would catch a total removal).
- ProseChars, graph.rs:1823: per character, at the work. Not seen: anything that is not per character (the blanked collection, the sort).
- ContractLines, graph.rs:1851: per line, but `code_spans(&paragraph)` and `paths_in_span` (graph.rs:1853-1855, 1866+) are uncounted. Re-reading the accumulated paragraph on every line (quadratic in a long paragraph) is not seen.
- ClaimChecks, graph.rs:1347/1359/1367: the two builds tick per entry via `.inspect` (good: rebuilding per claim with the inspect would be seen), the lookup ticks once per slug but the `loaded.contains` / `covered.contains` cost is not measured. Mutant M3 (index replaced by a scan) passes. Same for `covered`.
- ConditionsScan, rationale.rs:170: `.inspect` on the iterator that does the search, at the work. Good. Not seen: the `body.headings().iter().filter(lines.contains)` pass at rationale.rs:263.
- ProseLinesScanned, rationale.rs:185: `.inspect` placed after `take_while`, so it counts only lines inside the range. The positioning of the range start (`partition_point`, rationale.rs:181) is outside the count. Mutant M2 passes.
- HeadingsScanned, rationale.rs:219/283: per heading in the loop, at the work. Not seen: the lookup of the Explanation section (`sections().iter().find`) and the heading filter.
- NamingLookups, assessment.rs:635: one tick on entry to `naming`. The `named.contains(rest)` calls in the loop (BTreeSet membership) are not counted; replacing the set by a linear scan of the section's paths, or the `::`-suffix loop going quadratic in a long path, is not seen. The test pins exact calls, not their cost.
- ScanBytes, tense.rs:189/202: per character and per skipped link destination, at the work. Mostly sound. Not seen: work that happens outside `word_spans` (line splitting, the `Word` construction).
- PhraseTests, tense.rs:164: one tick per `matches_at`. Cost inside the call (for example `words.get(at + offset)` replaced by `words.iter().nth(..)`) is not seen.
- OutcomeChecks, sources.rs:214: per item, at the work. The membership checks inside the item (`known.contains`, `unloaded.contains`) are not counted. Mutant M1 passes.
- SlugsListed, sources.rs:206: ticks at the build of `known` only. `known` lookups are uncounted (M1). The `unloaded` set is not counted at all.

Required fix, same shape for all: count at the lookup (a `Work::...Lookups` tick at each `contains`, as Task 1's follow-up did for `ListedLookups` in raf-okf), and assert the lookups equal the items, so replacing the index by a scan changes a counted quantity only if the scan itself is counted. Since a scan cannot be ticked from the outside, the stronger form is to tick inside a wrapper that owns the set (a small `Counted<Set>` whose `contains` ticks and whose scan-mutant would have to change the type), or to move `partition_point` to a counted helper. At minimum: count the `loaded`/`covered` lookups (graph.rs:1369,1371), the `known`/`unloaded` lookups (sources.rs:219-220), the `named.contains` calls (assessment.rs:640-647), and the prose-range start (rationale.rs:181).

## Other findings

- (a) Vacuity, tense test: crates/raf-workplan/src/obligation/tests.rs (`hostile_text_is_read_in_linear_time`, the growth loop) asserts `three + one <= 2 * two` with no `one > 0`. For the "solid" and "openers" shapes (`hits` minimum 0) the phrase count may legitimately be zero, and then `0 + 0 <= 0` passes with the tick removed. Add `assert!(one > 0)` where `per_unit > 0`, and an exact expectation for the shapes that test no phrase. Other counted tests are exact, or growth with `one > 0` (graph.rs blocks test; ProseChars is exact). graph.rs BlankedSteps has only an upper bound plus `one > 0`, acceptable.
- (b) cfg(test)-only: sound. Non-test lines are only `mod work;` (lib.rs), `add(..)` calls and `.inspect(|_| add(..))` adapters on iterators that already ran, and a no-op `#[inline] pub(crate) fn add`. `at - next` (tense.rs:202) cannot underflow. No visibility change beyond `pub(crate)`. Clippy with `-D warnings` on the lib target is clean.
- (c) Integration tests (records.rs, rationale.rs, assessments.rs, obligations.rs): every `Instant`, `Duration`, `BUDGET`, `RELEASE_BOUND`, `timed_check`, `within_bound` is gone, and no functional assertion is removed. `contract_with` now returns the result alone; all call sites were updated and keep their assertions. The old integration test `moves_to_unknown_records_cost_the_same_...` moved to a unit test in obligation/tests.rs and keeps the 8000-item functional assertion (`ItemWithoutOutcome` count). No weakening. Remaining `Duration`/`elapsed` hits in raf-workplan are product code in bound/evidence.rs, not tests. Stale wording: crates/raf-workplan/tests/assessments.rs:836 still says "found in bounded time" in a comment, and the test name `hostile_text_is_read_in_linear_time` remains.
- (d) Bindings and Verification text: three spot checks hold (rationale `a_hundred_thousand_headings_are_each_read_once`, obligation `moves_to_unknown_records_cost_the_same_with_few_records_as_with_many`, work `the_tallies_are_reset_between_measures` all exist at the bound path). The Verification claim in docs/capabilities/workplan-document-model.md that the claims "are judged against sets built once, with one look for each slug of each claim" overstates: the look is a tick, the lookup cost is not measured (see M3). The sentence "A count sees the steps that are counted" is honest but does not cover this. Reword with the fix.
- (e) Repo rules: no clock words, issue or PR numbers, or process wording in added text; "counted, not timed" describes the test, acceptable. SPDX present in the new crates/raf-workplan/src/work.rs. Note the module is a copy of raf-okf's work.rs (duplicate machinery in two crates); acceptable if intended, but a shared definition would remove the drift risk (raf-okf got `ListedLookups`, raf-workplan has no lookup kinds).

## Fix list for the next revision

1. Count lookups at the point of lookup: graph.rs `loaded`/`covered`, sources.rs `known`/`unloaded`, assessment.rs `named.contains`, rationale.rs range start; assert exact equality with items in the matching tests.
2. Add `one > 0` to the tense growth assertion.
3. Re-run mutants M1 to M3 and see each fail.
4. Fix the document-model Verification wording and the stale comment at tests/assessments.rs:836.

---

# Delta recheck at e9e8a87 (Task 2 fix) and 49b0fb9 (Task 3 coverage)

FINAL VERDICT: PASS

Scratch clone /tmp/t2rev2.cT9qyV/repo at e9e8a87; nothing in the worktree touched.

## First-pass mutants, re-run on HEAD

| Mutant | Result |
|---|---|
| M1 (sources.rs `known` as `Vec`, scanned) | Fails fast: compile error E0277, `Vec<&str>: Holds<_>` not implemented (the set is only reachable through `Counted`) |
| M2 (rationale.rs `partition_point` replaced by `iter().position(..)`, bypassing the closure) | Fails: `empty_conditions_headings_...` (rationale.rs:418) and `the_decisions_of_an_explanation_...` (rationale.rs:443) panic on `0 < starts` |
| M3 (graph.rs `loaded` as `Vec`, then a direct scan of `evidence` bypassing `Counted`) | Fails: `the_claims_of_many_capabilities_...` (graph.rs:2221) left 6000, right 12000 (the exact two lookups per slug) |

## New mutants

| Mutant | Result |
|---|---|
| N1 coverage.rs `exclusion_for`: `break` after the first match | Fails: 3 of 4 coverage tests (`every_exclusion_is_tested_against_every_entry_exactly_once`, and the two growth tests) on the exact product entries x exclusions |
| N2 scan hidden inside the `Counted` callers: add `impl Holds for Vec<T>` that scans, and make `known` a `Vec` | SURVIVES (23 obligation tests ok). Same for `loaded` as `Vec` with such an impl |
| N3 scan through the counted closure (not run; follows from the type): `iter().position` inside the `RangeStarts` closure would tick `lines` times and pass the `log2 + 4` bound only if binary, so it fails | covered by `search_steps` |

N2 is the one residual limit: a count of lookups cannot see the cost of one lookup. It needs a new `Holds` impl in work.rs, a visible edit to the counting machinery itself, and the lookup counts stay exact. Every other form of the documented kind (bypassing the set, looking up more or fewer times, building per item, scanning outside the set, scanning through the counted closure) is caught. I accept this as the inherent boundary and the capability text states what the counts pin; not blocking.

## Non-test behaviour and visibility

- `Counted`, `Holds` and the new `Work` kinds are not `cfg(test)`: they are `pub(crate)` types compiled in release too. `Counted::contains` calls the no-op `add` and then the set's own `contains`: same result, no output change, no public API change. Use sites (graph.rs add_claims, sources.rs items, assessment.rs `naming` closure, rationale.rs `has_item`, tense.rs `matches_at`, graph.rs `code_spans`) change only by the count calls and the wrapper. Moving `add(PhraseTests)` into the per-word loop is equivalent for behaviour.
- `cargo build -p raf-workplan --release --locked`: finished with no warnings.
- Minor: shipping a wrapper type in non-test builds is a small footprint; wrapping would be cleaner under `cfg(test)`, not a blocker.

## Gates

- `cargo fmt --all --check` clean; `cargo clippy -p raf-workplan --locked --all-targets -- -D warnings` clean.
- `cargo test -p raf-workplan --locked --lib -- work:: graph:: rationale:: assessment:: obligation:: coverage::` 56 passed.

## Task 3 (49b0fb9) note

`EntriesJudged` ticks once per entry in `Judge::judge` and `ExclusionTests` once per exclusion tested in `exclusion_for`; tests pin the exact product (entries x exclusions, 0 with no exclusions), growth in both axes with `one > 0`, and the report's own counts. Cost inside `matches` (path iteration) is not counted; same inherent limit as N2. Tense growth test now requires a nonzero count where a phrase can be tested (first-pass finding (a) addressed). The stale comment at tests/assessments.rs:836 was not rechecked.
