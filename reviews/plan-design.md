# Plan design review (SOLID / ETHOS)

Reviewed plan: `.stage/index-the-coverage-product-instead-of/plan.md` (Rev 0)

## Code read

- `crates/raf-workplan/src/coverage.rs`: `matches` (781-789), `exclusion_for` (792-806),
  `ancestors` (759-761), `Bound::of`/`covers` (707-756), `Judge` (832-943), `judge` (845-869),
  `judge_item` (873-895), `surface` (899-942), `compute` (1054-1129), `check` (563-606),
  `findings` (611-689).
- `crates/raf-workplan/src/coverage/tests.rs` (whole file): the `judged` harness and the four
  counted tests.
- `crates/raf-workplan/src/work.rs`: `Work` (20-59), `Counted`/`Holds` (62-107), `KINDS`/`ALL`
  (110-135), `measure_kind` (198-204).
- `crates/raf-okf/src/profile/typed.rs:693-721` (`ExclusionItem`, `Exclusion`),
  `crates/raf-okf/src/profile/records.rs:51-65, 448-461` (`path_segments`, `exclusion_item`).
- `docs/capabilities/workplan-two-way-coverage.md:35-41, 242-261, 289-292`.

## What the index must reproduce (established from code)

- `ExclusionItem::Item(p)` matches an entry iff some path of `matching` equals `p` exactly
  (coverage.rs:783). `ExclusionItem::Prefix(p)` matches iff some path is `p` followed by `::`
  (coverage.rs:784-787): `p` never matches itself, and `a::b` never matches `a::bc`.
- Exclusion paths are identifier segments joined by `::` with two or more segments
  (records.rs:51-65, 453-458), so a prefix never ends in `:` and the set of prefixes a path
  can match is exactly `ancestors(path).skip(1)` (coverage.rs:759-761). The plan's "prefix text
  before each `::` boundary" is that iterator.
- `matching` per entry (the paths the index is probed with): a public item gives every path it
  is exported under in the target plus its defining path (coverage.rs:892-893, often the same
  string twice); an unresolved re-export gives its exported path plus each path its written
  path names from its declaring module(s) (915-916); a test gives its path twice (1104-1110 then
  892-893). So lookups per entry are bounded by the segments of a handful of paths, independent
  of the exclusions, as the plan's R1 check assumes.
- Every matching exclusion is marked in `matched` (coverage.rs:801) and the first in record
  order is returned (802); `findings` reads `matched` zipped with the record (674).
- There is one production caller of `exclusion_for` (`Judge::judge`, 847) and one site that
  builds `Judge` (`compute`, 1080). The design therefore has one place to put the index.

The design (two maps keyed by exclusion text, values the record indices, probe with the exact
path and its ancestors, mark all, report the minimum index) reproduces these semantics, and the
R3 test list plus the mutant list cover the ways it could diverge. No correctness, silent
failure or duplicate production path finding. The findings below are change-cost and
verifiability.

## Findings

### ADVISORY 1: the R2 "linear-growth check" as designed also passes for the product

Plan: "doubling entries at fixed exclusions at most doubles total work; doubling exclusions at
fixed entries at most doubles total work (`one>0, three+one<=2*two` idiom of the repo)".

Consequence: `entries x exclusions` with one factor fixed is linear in the other; the existing
product tests already assert `three + one <= 2 * two` and pass with equality
(tests.rs:174-176, 193-195). A check of this shape cannot tell the index from the product it
replaces, so R2 as written verifies nothing the old tests did not. (R1's "same entries, 10x the
exclusions: same lookups" and `ExclusionsIndexed == exclusions` do discriminate; R2 should too,
since it is the named deliverable.)

REWRITE TO: scale entries and exclusions together: for `units` in 1..=3 use
`units * MODULES_PER_UNIT` modules and `units * EXCLUSIONS_PER_UNIT` exclusions, and assert
`three + one <= 2 * two` on the sum `EntriesJudged + IndexLookups + ExclusionsIndexed` (the
product gives 1, 4, 9 units and fails; the sum passes). Keep the two single-axis checks as
they are if wanted, but say which kind each measures (lookups for the entries axis,
exclusions indexed for the exclusions axis).

### ADVISORY 2: the prefix probe should be `ancestors`, not a new boundary walker

Plan: "for each `::` boundary in the path, the prefix text before it looked up in `prefixes`".

Consequence: `ancestors` (coverage.rs:759-761) already yields exactly those texts and is the
walker `Bound::covers` uses for the same "is some ancestor in a set" question
(coverage.rs:753, 919). A second walker (for example `match_indices("::")`) is a second way to
enumerate a path's modules; if the two ever differ (the re-export paths at 915-916 are built
from written text, not index paths) coverage and exclusion would disagree on what "below a
module" means.

REWRITE TO: probe `items.get(path)` then `ancestors(path).skip(1)` against `prefixes`;
name `Bound::below`/`covers` as the precedent in the plan. Counted work is then
`paths x ancestors(path).count()`, which is the plan's `1 + boundaries`.

### ADVISORY 3: put the index, the record slice and `matched` in one type with one constructor

Plan: "`ExclusionIndex { items: HashMap<&str, Vec<usize>>, prefixes: HashMap<&str, Vec<usize>> }`
over the exclusions (indices in record order)" alongside `Judge.exclusions` and `Judge.matched`.

Consequence: the stored `usize`s index both `exclusions` and `matched`; today the invariant
"same length, same order" is held by one line in `compute` (coverage.rs:1084) and the `zip`
in `exclusion_for` would silently truncate if it broke. With three fields on `Judge` sharing it,
the next change to `compute` can build them inconsistently and the index will panic on
`matched[i]` or mark the wrong exclusion.

REWRITE TO: `struct ExclusionIndex<'r> { exclusions: &'r [Exclusion], items, prefixes,
matched: Vec<bool> }` with `fn new(exclusions: &'r [Exclusion]) -> Self` (counts
`ExclusionsIndexed`) and `fn first_match(&mut self, paths: &[&str]) -> Option<&'r Exclusion>`
(counts `IndexLookups`, marks all, returns the minimum index) and
`fn into_matched(self) -> Vec<bool>`. `Judge` holds one `ExclusionIndex`; `compute` destructures
it as it does `matched` today (1113-1118).

### ADVISORY 4: say that the linear scan leaves production; the oracle lives only in tests

Plan: "a differential test: the indexed result equals a reference product implementation on
seeded random exclusions/paths (reference kept in the test module)".

Consequence: the plan replaces the counter but never says `matches` (781-789) and the
slice-walking `exclusion_for` (792-806) are deleted. If they stay (dead or called from the
test oracle), there are two production matchers and a future reader fixes the wrong one.
Separately, the workspace has no random-number dependency; a seeded generator written for
one test is more code than the case space needs.

REWRITE TO: delete `matches` and `exclusion_for`; the oracle in `coverage/tests.rs` is the
only linear scan. Replace "seeded random" by an enumerated cross product: exclusions drawn from
`{Item(a::b), Item(a::b::c), Prefix(a::b), Prefix(a), Item(a::b) again, Prefix(a::bc)}` in
several orders against paths `{a::b, a::b::c, a::bc, a::bc::d, a::b::c::d}`, asserting the
returned exclusion and the full `matched` vector agree with the oracle for every order. That
covers item-vs-prefix at the same text, the sibling case, the duplicate, and first-in-record.

### ADVISORY 5: the change set names three tests; four are bound and one more counter site

Plan: "the three product tests are replaced"; "replace `ExclusionTests` by the two new kinds in
`work.rs`".

Consequence: `without_exclusions_the_entries_are_judged_and_nothing_is_tested` (tests.rs:150)
reads `Work::ExclusionTests` and is bound at workplan-two-way-coverage.md:41 and 292; the
harness `judged` limits `Work::ExclusionTests` (tests.rs:113); `KINDS` is 19 and `ALL` lists
every kind (work.rs:111-135); the test module doc (tests.rs:4-8) states the product. A renamed
or removed bound test without the binding change is a `check` failure on the capability.

REWRITE TO: list all four tests and their new names, the harness's limited kind (`IndexLookups`,
with `ExclusionsIndexed` asserted from the tally), `KINDS = 20` plus the `ALL` entries, and
the two doc sites (tests.rs module doc; capability Verification text and both binding lists).

## Verdict

The design is one path (index built once in `compute`, probed once per entry), reproduces the
exact `Item`/`Prefix` semantics and the first-in-record rule, keeps every match marked so
`coverage-exclusion-unmatched` is unchanged, and is checkable through counted work. The
advisories tighten what R2 proves, reuse `ancestors`, and put the shared invariant in one
constructor.

VERDICT: PASS
