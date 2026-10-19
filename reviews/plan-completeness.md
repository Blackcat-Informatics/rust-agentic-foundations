# Plan completeness review

Reviewed plan: `/home/paudley/Active/rust-agentic-foundations/.worktrees/29-index-the-coverage-product-instead-of/.stage/index-the-coverage-product-instead-of/plan.md` (Rev 0)

Inputs read in full: issue.md (body, 0 comments), issue-analysis.md (R1-R6, 2 NEEDS-JUDGMENT),
prior-art-assessment.md (3 NEEDS-JUDGMENT), plan.md, `crates/raf-workplan/src/coverage.rs`,
`crates/raf-workplan/src/coverage/tests.rs`, `crates/raf-workplan/src/work.rs`,
`docs/capabilities/workplan-two-way-coverage.md` (bindings and Verification),
`crates/raf-workplan/tests/coverage.rs` (the two existing exclusion tests the plan cites),
`crates/raf-okf/src/profile/records.rs` (`check_exclusions`), `CHANGELOG.md`.

## Requirement -> task -> check

| Ledger R | Requirement (verbatim) | Plan row / task | Check (as planned) | Status |
|---|---|---|---|---|
| R1 | "Build an index so the work grows with entries plus exclusions" | plan R1 / Task 1 | `ExclusionsIndexed == exclusions`; same entries, 10x exclusions -> same `IndexLookups`; lookups "bounded by the segments of the entries' paths" | MAPPED. The equalities can be falsified; the "bounded by the segments" bound is not stated as a formula (F3, advisory) |
| R2 | "replace the exact product pin with a linear-growth check." | plan R2 / Task 1 | grow entries at fixed exclusions, grow exclusions at fixed entries, `one>0, three+one<=2*two` on total work | UNFALSIFIABLE: entries x exclusions passes both checks (F1, BLOCKING) |
| R3 | "`exclusion_for` marks every exclusion that matches an entry, not only the first one it keeps" | plan R3 / Task 1 | new test: an exclusion whose only matches are entries another exclusion reports first is not reported unused; item and prefix exclusions on one entry both marked; differential test against a reference product | MAPPED. The new tests do not name the entry point that shows "marked" (F4, advisory) |
| R4 | "An index must still return every matching exclusion for each entry" | plan R3 / Task 1 | differential test (indexed result equals the reference product: first match and the `matched` vector) on seeded inputs | MAPPED |
| R5 | "the report of unused exclusions must stay the same" | plan R4 (and R3) / Task 1 | the existing `tests/coverage.rs` passes unchanged, including `an_exclusion_without_a_reason_or_that_matches_nothing_fails_naming_the_file` and `the_first_matching_exclusion_gives_the_reason` (both go through `Workplan::check` and look for `coverage-exclusion-unmatched`) | MAPPED: real entry point, specific finding kind |
| R6 | "an index that stops at the first match would hide unused exclusions." | plan R3 / Task 1, Mutants line | mutant "index stops at the first match" must fail the R3 test | MAPPED |
| NJ1 (ledger) | what the linear-growth check asserts | Ledger mapping paragraph | "the repository idiom (`one > 0`, `three + one <= 2 * two` on counted work)" | DISPOSED, but the chosen idiom cannot falsify the product (F1) |
| NJ2 (ledger) | reproduce `exclusion_for` exactly, or only the unused report | Ledger mapping paragraph | exactly: the first match in record order gives the reason, every match is marked | DISPOSED |
| PA-NJ1..3 | file location; PR 30 attempt or precedent; no named artifacts | NEEDS-JUDGMENT dispositions | file is `coverage.rs`; PR is precedent; plan names the files | DISPOSED |

## Findings

### F1 (BLOCKING, UNFALSIFIABLE): the linear-growth check passes for the product it replaces

Plan line (contract R2): "the three product tests are replaced by growth checks: doubling entries at fixed exclusions at most doubles total work; doubling exclusions at fixed entries at most doubles total work (`one>0, three+one<=2*two` idiom of the repo)".

`entries x exclusions` is linear in each factor when the other is held fixed. The current tests
`the_exclusions_tested_grow_with_the_entries_and_no_faster` and
`..._grow_with_the_exclusions_and_no_faster` already pass `three + one <= 2 * two` with the
product. For example, with exclusions fixed at E, entries are 230, 340 and 450 units:
450E + 230E = 2 * 340E. The planned check therefore cannot tell the index from the product the
issue asks to remove. The NJ1 disposition repeats this choice. The R1 equalities on the new
counters catch some mutants. They do not make R2's "linear-growth check" a check that the work
grows with entries **plus** exclusions.

REWRITE TO (contract R2 Check):
"the four product-pinning tests (`every_exclusion_is_tested_against_every_entry_exactly_once`, `the_exclusions_tested_grow_with_the_entries_and_no_faster`, `the_exclusions_tested_grow_with_the_exclusions_and_no_faster`, `without_exclusions_the_entries_are_judged_and_nothing_is_tested`) are replaced. `judged` runs `plan.coverage_report()` under `measure` (every kind held) with the ceiling `K * (entries + exclusions)` for a stated constant K, so any per-entry scan of the exclusions panics. New test `the_work_grows_with_entries_plus_exclusions`: for units 1..=3, scale entries (`units * MODULES_PER_UNIT` modules) and exclusions (`units * EXCLUSIONS_PER_UNIT`, half `item` and half `<module>::*`) together. Total = `EntriesJudged + IndexLookups + ExclusionsIndexed`. Assert `one > 0` and `three + one <= 2 * two`. A product gives 1:4:9 and fails. The two one-factor growth tests stay as well. Mutant: put back the per-entry scan over all exclusions, counted as `IndexLookups`; the joint test and the ceiling must fail."

Also rewrite the NJ1 disposition. REWRITE TO: "'linear-growth check' means the repository idiom (`one > 0`, `three + one <= 2 * two`) applied to total counted work while entries and exclusions are scaled **together**, plus a `measure` ceiling linear in entries + exclusions. One-factor scaling alone does not separate a product from a sum."

### F2 (BLOCKING, STRUCTURE): Task 1 has no focused validation and no issue update

Plan line: "Independent review, commit, push."

The method requires each task to have focused validation, an independent review, a commit, a
push and an issue update. Task 1 names no validation command and no issue update.

REWRITE TO (end of Task 1): "Focused validation: `cargo test -p raf-workplan --lib coverage::tests`, `cargo test -p raf-workplan --lib work::tests`, `cargo test -p raf-workplan --test coverage`, and the mutants listed under the contract, each run in a scratch clone and each failing a named test. Then an independent task review, commit, push, and an issue comment saying what landed and naming the replaced and added tests."

### F3 (ADVISORY, check precision): the lookup bound is prose

Plan line (contract R1): "`IndexLookups` bounded by the segments of the entries' paths".

The design already gives the count: one lookup per path plus one per `::` boundary, which is
the path's segment count. Each matching path of an entry is the exported paths plus the
defining path, which `judge_item` pushes even when it is the same as an exported path.

REWRITE TO: "`IndexLookups ==` the sum over every judged entry of the segment counts of its matching paths (computed in the test from the generated crate: `big::m{m}::f{i}` twice, `big::m{m}` twice, `big::test_crate::it::t{m}_{i}` twice). The value is the same for 1x and 10x the exclusions."

### F4 (ADVISORY, entry point): the new R3 tests do not say where "marked" is observed

Plan line (contract R3): "tests: an exclusion that matches only entries whose first match is another exclusion is NOT reported unused; ... an item exclusion and a prefix exclusion that both match one entry both marked".

`matched` is visible only through `Workplan::check`, as the absence of
`coverage-exclusion-unmatched`. `coverage_report` does not expose it.

REWRITE TO: "tests in `crates/raf-workplan/tests/coverage.rs` through `Workplan::check`: (a) exclusions `X::m::*` then `X::m::f`, where `f` is the only item below `m` matched by the second: `check` gives no `coverage-exclusion-unmatched`, and the report's reason for `X::m::f` is the first one's; (b) an `item` exclusion and a `<module>::*` exclusion matching one entry: no `coverage-exclusion-unmatched` for either."

### F5 (ADVISORY, TEST-ONLY): duplicate exclusions cannot reach the shipped entry point

Plan line (Design): "Duplicate exclusions (same item twice) keep both indices (both are marked, the first reports)". Contract R3 lists "duplicates;" with no expected result.

`raf_okf::profile::records::check_exclusions` refuses a repeated `item` text
(`Rule::DuplicateEntry`, `valid = false`), so a record with duplicates never loads and
`check` never judges it. As written, "duplicates;" states no expected result and names no input
that can reach judging.

REWRITE TO: "duplicates: through `Workplan::check`, a record that repeats an exclusion is refused at load (`duplicate-entry`). In the module's differential test only, a hand-built `&[Exclusion]` with a repeated item marks both and returns the first."

### F6 (ADVISORY, scope detail): fourth test, module doc, changelog

- The plan says "replace the three product tests". `without_exclusions_the_entries_are_judged_and_nothing_is_tested` also reads `Work::ExclusionTests` and is bound in the capability (bindings line 41, table line 292). Name its replacement and its expected counts with no exclusion (`ExclusionsIndexed == 0`; `IndexLookups` either 0 because an empty index short-circuits, or the segment sum). The rewrite in F1 covers this.
- The module doc of `coverage/tests.rs` (lines 4-8: "each exclusion of the record is tested against each of them") states the product. Add it to Task 1's edit list.
- "the changelog if the repository keeps one for raf-workplan": the repository keeps `CHANGELOG.md` with per-crate bullets under `[Unreleased]`. REWRITE TO: "add a `**raf-workplan:**` bullet under `[Unreleased]` in `CHANGELOG.md`: coverage judges exclusions through an index, so its work grows with entries plus exclusions."

## Not a finding (for the final reviewer)
`check` builds `needed` by looping over exclusions x targets (coverage.rs lines 589-596). The
issue names only the entry-by-exclusion judging, so this loop is outside the requested scope.

VERDICT: NEEDS-REVISION
