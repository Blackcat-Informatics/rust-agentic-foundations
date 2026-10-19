# Plan completion adversary review

- Reviewed plan: `/home/paudley/Active/rust-agentic-foundations/.worktrees/29-index-the-coverage-product-instead-of/.stage/index-the-coverage-product-instead-of/plan.md` (Rev 0)
- Pass: 1 (single pass; no recheck)
- Inputs read: plan.md, issue.md (body, 0 comments), issue-analysis.md, prior-art-assessment.md, brief.json, AGENTS.md (gates), CLAUDE.md, `crates/raf-workplan/src/coverage.rs`, `crates/raf-workplan/src/coverage/tests.rs`, `crates/raf-workplan/src/work.rs`, `crates/raf-workplan/tests/coverage.rs`, `crates/raf-okf/src/profile/records.rs`, `docs/capabilities/workplan-two-way-coverage.md`, `work/coverage/default.md`, `CHANGELOG.md`. No `.goals`, `.baseline` or `CONSTITUTION.md` exists in the worktree.
- User scope decisions quoted by the caller: none. The issue has no comments. Nothing below changes scope.

The scope matches the issue. Every ledger row (R1-R6) maps to a plan row, nothing the issue asks for is declined, and no refusal is counted as delivery. The findings below are about criteria that would not tell success from the current code, and about one task that does not cover all the tests it removes support for.

## F1 (high): UNFALSIFIABLE CRITERION. The R2 "linear-growth check" passes on the product it is meant to replace

Plan line (contract row R2):

> | R2 | Replace the exact product pin with a linear-growth check | 1 | the three product tests are replaced by growth checks: doubling entries at fixed exclusions at most doubles total work; doubling exclusions at fixed entries at most doubles total work (`one>0, three+one<=2*two` idiom of the repo) |

Evidence: the current product tests already assert exactly this idiom on the product work, and they pass in today's gate.
`crates/raf-workplan/src/coverage/tests.rs:174-176` (entries scaled, exclusions fixed) and `:193-195` (exclusions scaled, entries fixed) both assert `one > 0` and `three + one <= 2 * two` on `entries * exclusions`. When one factor is held fixed, the product is affine in the other, so the idiom holds with equality. The proposed R2 check therefore cannot tell the index from the product. It also passes for the plan's own mutant "the index built per entry", because E·X is affine in each variable on its own. Only R1's "10x the exclusions: same lookups" clause catches either one. The issue asks that "the work grows with entries plus exclusions". To separate E + X from E·X, both must grow together.

There is a second hole. Counted work only sees what calls `add`. Nothing in the plan rules out an uncounted per-entry pass over `exclusions` (the current `exclusion_for` loop, or `matches`) left in the production path next to the counted index.

REWRITE TO:
> | R2 | Replace the exact product pin with a linear-growth check | 1 | In `coverage::tests`, `every_exclusion_is_tested_against_every_entry_exactly_once`, `the_exclusions_tested_grow_with_the_entries_and_no_faster` and `the_exclusions_tested_grow_with_the_exclusions_and_no_faster` are removed. Their replacements all drive `Workplan::coverage_report` through the `judged` helper and use T = `EntriesJudged + IndexLookups + ExclusionsIndexed`. (a) A joint-growth test scales entries and exclusions together for u = 1, 2, 3 (`modules = u * MODULES_PER_UNIT`, `exclusions = u * EXCLUSIONS_PER_UNIT`) and asserts `one > 0` and `three + one <= 2 * two` on T. The product (T = E + E·X, quadratic in u) fails this; the index (T affine in u) passes it. (b) The two one-variable growth tests stay, with the same idiom applied to T. (c) In the scratch clone, the joint-growth test is run against the pre-change `exclusion_for`, with its per-pair `add` counted as `IndexLookups`, and it fails. The handoff note records that failure. (d) No function in `crates/raf-workplan/src/coverage.rs` iterates `exclusions` once per entry. The `exclusion_for` loop over all exclusions and `matches` leave the production path; the reference product exists only in `coverage/tests.rs`. The only per-entry access to exclusions is the index lookup, and it counts `IndexLookups`. |

## F2 (medium): UNMAPPED / INSUFFICIENT TASK. A fourth bound test reads the removed work kind, and the plan contradicts its bound claim

Plan lines:

> Task 1: ... replace `ExclusionTests` by the two new kinds in `work.rs`; replace the three product tests; ...

> R1 ... `IndexLookups` bounded by the segments of the entries' paths and independent of the number of exclusions (same entries, 10x the exclusions: same lookups)

Evidence: `without_exclusions_the_entries_are_judged_and_nothing_is_tested` (`coverage/tests.rs:149-158`) is a fourth test. It reads `Work::ExclusionTests` through `judged` (`:113`, `:122`) and holds that kind to a ceiling of 0. It is bound by the capability (`docs/capabilities/workplan-two-way-coverage.md:41`, generated matrix `:292`). The bound Verification sentence "with no exclusion nothing is tested" (`:261`) depends on it. The plan names only "the three product tests" and never says what becomes of this test or that sentence. Read literally, "independent of the number of exclusions" means zero exclusions make the same non-zero number of lookups, which makes the sentence false. The test module's own doc (`coverage/tests.rs:4-8`, "every exclusion must be tested against every entry") also turns false, and the plan does not list it.

REWRITE TO (append to Task 1, and narrow the R1 clause):
> `without_exclusions_the_entries_are_judged_and_nothing_is_tested` stays bound and is rewritten to the new kinds. With zero exclusions no index is built and no lookup is made: under `measure_kind(Work::IndexLookups, 0, ..)` it asserts `IndexLookups == 0`, `ExclusionsIndexed == 0` and `EntriesJudged` equal to the entries. The Verification sentence "with no exclusion nothing is tested" (workplan-two-way-coverage.md:261) stays and stays true. The R1 clause reads: "for the same entries, X and 10·X exclusions (X ≥ 1) give the same `IndexLookups`". The module doc of `coverage/tests.rs` (lines 4-8) is rewritten to state the index bound in place of the product.

(If the planner instead drops the zero-exclusion shortcut, the rewrite is: the test asserts that `IndexLookups` with zero exclusions equals `IndexLookups` with X ≥ 1 for the same entries, line 261 of the capability loses "and with no exclusion nothing is tested", and the test is renamed and re-bound. One of the two options must be stated. Silence is not an option.)

## F3 (medium): UNFALSIFIABLE CRITERION. R4 "report unchanged" is self-contradictory and partly prose

Plan line (contract row R4):

> | R4 | The report is unchanged | 1 | the existing coverage test suites and `tests/coverage.rs` pass unchanged; the capability text's statements about reasons/unused stay true |

"The existing coverage test suites ... pass unchanged" contradicts Task 1, which rewrites `coverage/tests.rs`. "The capability text's statements ... stay true" is a judgment, not an observation. This repository's own record has `exclusions: []` (`work/coverage/default.md:7`), so running the CLI on this repository exercises no exclusion and would prove nothing about the report.

REWRITE TO:
> | R4 | The report is unchanged | 1 | `crates/raf-workplan/tests/coverage.rs` has an empty diff against the base commit and every test in it passes. Among them, `the_first_matching_exclusion_gives_the_reason`, `an_exclusion_without_a_reason_or_that_matches_nothing_fails_naming_the_file`, `a_prefix_matches_below_its_module_and_never_a_sibling_that_shares_its_text`, `an_item_is_excluded_by_its_exported_or_its_defining_path` and `a_unit_test_module_is_excluded_by_its_prefix_with_the_reason_shown` drive item and prefix exclusions through `Workplan::coverage_report` and `Workplan::check`. The Contract sentences of `docs/capabilities/workplan-two-way-coverage.md` on the first matching exclusion giving the reason and on `coverage-exclusion-unmatched` (lines 130-135) have an empty diff. The only tests that change are those in `coverage/tests.rs` that read the removed `Work::ExclusionTests`. |

## F4 (low-medium): UNFALSIFIABLE CRITERION. The "duplicates" item has no observation and targets input the loader refuses, which leaves the marking work uncounted

Plan lines:

> Duplicate exclusions (same item twice) keep both indices (both are marked, the first reports).

> R3 ... duplicates; ...

Evidence: the record profile refuses a repeated exclusion text when the record loads (`Rule::DuplicateEntry`, `crates/raf-okf/src/profile/records.rs:503-511`). `check` then finds `coverage:default` present but not loaded and judges no coverage (`coverage.rs:564-566`), so no shipped entry point can hand `compute` two exclusions with one key. As written, "duplicates" names no observation and could only be shown by a test-only call on a hand-built slice. It also hides a gap in the R1 claim. The design keeps a `Vec<usize>` per key and "mark[s] every index", and no counted kind covers that walk. The claim that work grows with entries plus exclusions rests only on `IndexLookups` and `ExclusionsIndexed`, so it holds only if every index walked is bounded by a counted kind.

REWRITE TO:
> Design: duplicate exclusion texts are refused when the record loads (`duplicate-entry`), so in a loaded record each key of `items` and of `prefixes` names one exclusion. Every exclusion index walked while marking is counted, under `IndexLookups` or under its own kind included in R2's T, so the marking cannot add uncounted work. R3 test (replacing "duplicates"): a record that writes one exclusion text twice gives the `duplicate-entry` finding through `Workplan::check` and no coverage finding, so no index ever holds two exclusions for one key.

## F5 (low): UNFALSIFIABLE CRITERION. The new R3 tests name neither an entry point nor what the differential test compares

Plan line (contract row R3, excerpts):

> an exclusion that matches only entries whose first match is another exclusion is NOT reported unused; ... a differential test: the indexed result equals a reference product implementation on seeded random exclusions/paths (reference kept in the test module)

"Reported unused" is a `Workplan::check` finding, but the plan never says that, so asserting on internal `matched` state would also pass it. "The indexed result" could be read as the returned first exclusion alone. A differential test that only compares the returned exclusion would miss the defect the issue warns about (ledger R6: "an index that stops at the first match would hide unused exclusions").

REWRITE TO:
> R3 tests: (i) through `Workplan::check`, with the record `[made::body::*, made::body::f]`, where only the entry `made::body::f` matches the second exclusion: no `coverage-exclusion-unmatched` finding names `made::body::f`, and the `Excluded` row of `Workplan::coverage_report` for `made::body::f` carries the reason of `made::body::*`. (ii) A differential test with a fixed seed and a stated case count. For every generated entry, the index and the reference product in `coverage/tests.rs` return the same first exclusion index. After all entries, the two `matched: Vec<bool>` are equal. Both outputs are compared.

## Observations (not completion findings; for the other reviewers)

- Task 1 says "the changelog if the repository keeps one for raf-workplan". That condition is already settled: `/CHANGELOG.md` exists, with crate-tagged bullets under `## [Unreleased]`, `raf-workplan` among them. The plan can name the entry outright and drop the "if".
- `check` computes the needed packages by testing every exclusion against every target (`coverage.rs:589-596`, exclusions × targets). That is not the entries × exclusions product the issue names, so leaving it alone is not a descope. A one-line scope statement in the plan would stop R1's claim from being read as covering all coverage work.
- Task 2, "`make check` minus cross-check locally (hosted CI covers it)", is a matter for the constraint reviewer. AGENTS.md:175 says "A missing tool is a hard failure, not a skip", and CLAUDE.md's platform rule says a platform-specific failure does not block a change.

## Required user decisions

None. Every finding can be fixed by rewriting the plan text. No scope or criterion choice here needs the user.

VERDICT: FINDINGS-OPEN
