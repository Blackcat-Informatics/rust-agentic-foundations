# Enhancement audit: plan.md (Rev 0), index the coverage product

Reviewed plan: `/home/paudley/Active/rust-agentic-foundations/.worktrees/29-index-the-coverage-product-instead-of/.stage/index-the-coverage-product-instead-of/plan.md`

Standing constraints: no `.goals` file in the worktree; CLAUDE.md/AGENTS.md apply (no timing
runs, no tracker numbers in committed files, no deferrals, no weakened lints). No item below
conflicts with them. No web tools were used; the audit is from the codebase and first principles.

## Items (transformation first)

### 1. TRANSFORMATION: one path-order index for every annotated path set of the module, not an exclusion-only index

**The concept the feature is a shadow of.** `coverage.rs` holds three sets of annotated nodes
in the tree order on paths, all queried by the same ancestor walk:

| Set | Annotation | Match rule | Today |
|---|---|---|---|
| `Bound.below` (coverage.rs:702, 753, 919) | "covered" | every path strictly below | `BTreeSet` + `ancestors()` walk, **uncounted** |
| `ExclusionItem::Prefix` | reason, record index | every path strictly below | linear scan over all exclusions |
| `ExclusionItem::Item` | reason, record index | the node itself | linear scan over all exclusions |

The plan indexes only the last two and leaves `Bound::covers` as it is. The definitive
structure is a single private `PathIndex<T>` (a segment trie, or the plan's hashed-prefix maps,
either gives O(boundaries) per query) with two annotation modes, `At` and `Below`, that returns
every hit in insertion order with its annotation. `Bound` becomes `PathIndex<()>` over the
bound module paths (plus the `ItemId` set it already keeps); the exclusions become
`PathIndex<usize>` over record indices. `matches`, `covers` and the sibling rule
(`a_prefix_matches_below_its_module_and_never_a_sibling_that_shares_its_text`) are then
implemented once and tested once.

**Why it is decisive, not cosmetic.**
- The plan's R2 check, "doubling entries at fixed exclusions at most doubles total work", is
  measured only over the new kinds; the `covers` ancestor walk does no `add` at all, so half of
  judging is invisible to the growth checks. With one index, every lookup of judging goes
  through one counted path and the linear claim covers the whole of `Judge::judge`.
- It is what the next issues in this area need: (a) report attribution ("which capability
  covers this item": the `Below` hit on `Bound` already knows the binding, so `Excluded`-style
  rows for covered items cost nothing); (b) glob exclusions such as `a::*::tests::*`
  (a trie node with a wildcard child, impossible on a per-boundary hash map); (c) the shadowed
  exclusion diagnostic of item 2, which needs all hits in order from one structure.
- The alternative weighed and not recommended: resolving exclusions through the source index
  as bindings are (`Bound::of` + `resolve_in_package`), so an unmatched exclusion is simply one
  that resolves to nothing. It would unify exclusions with bindings but it does not reproduce
  the current semantics (exclusions match exported, defining and test paths and the paths of
  re-exports the index cannot follow), so it endangers R4. The path index keeps the semantics
  exactly and still unifies the mechanism.

Cost: **M** (a private module `coverage/index.rs` or `path_index.rs`, ~150 lines plus its
unit tests; `Bound` rewired; no public API change). Payoff: decisive (linear and counted end to
end, one matching semantics, the structure the follow-on work needs).
Where: Design, "Build once per judging a `ExclusionIndex { items: HashMap<&str, Vec<usize>>,
prefixes: HashMap<&str, Vec<usize>> }`"; Declined, "A shared index crate: one consumer" (there
are two consumers in this file; the answer is a shared module, not a crate).

### 2. TRANSFORMATION: exclusion effectiveness, not a boolean

The constraint of the issue ("mark every exclusion that matches so an unused one can be
reported") is the degenerate case of exclusion effectiveness. Since the index must enumerate
every match of an entry anyway, counting is free: for each exclusion keep `matches: usize`
(entries it matched) and `reasons: usize` (entries it gave the reason for). Then

- unused: `matches == 0` (R5: the report is unchanged);
- **shadowed**: `matches > 0 && reasons == 0`, an exclusion every one of whose matches gets its
  reason from an earlier exclusion. Today such an exclusion is invisible: it appears in no
  `Excluded` row and is not `coverage-exclusion-unmatched`. It is exactly the dead entry that
  accumulates as `Prefix` exclusions are added above older `Item` exclusions.

Two tiers. Tier A (S): replace `Computed.matched: Vec<bool>` by the two counts and expose them on
`CoverageReport` (an `exclusions()` accessor with the written exclusion, its reason, `matches`
and `reasons`), rendered as a short table after `excluded`. Tier B (M): a finding
`coverage-exclusion-shadowed` (closed finding-kind list, capability text, tests), run first
against this repository's own `work/coverage/default.md` so the gate does not turn red on the
day the finding is added. Payoff: catches dead record entries, which the issue's own rationale
(report unused exclusions) is about. Where: Design, "mark every index in `matched`"; R3 row.

### 3. LEVERAGE: generate the prefix lookups with the existing `ancestors()` (coverage.rs:759)

`ancestors(path).skip(1)` is precisely "the prefix text before each `::` boundary" and it is
what `Bound::covers` already uses, so both walkers split a path at the same places (every
`::` occurrence, via `rsplit_once`, not a segment parser). Writing a second boundary walker
invites a divergence between the index and `matches` on any path whose `::` are not all module
boundaries. Cost: S. Payoff: one boundary semantics, fewer mutants to worry about. Where: Design,
"for each `::` boundary in the path, the prefix text before it looked up in `prefixes`".
(Subsumed by item 1 if adopted.)

### 4. LEVERAGE: count through `Counted`, not hand-placed `add` calls

The repository's idiom (work.rs:85-107; graph.rs:1345; obligation/sources.rs:202) is that the
count is part of the lookup so an uncounted lookup cannot exist. `Counted` only wraps sets via
`Holds`; add a map form (`Counted<HashMap<K, V>>::get` or a `Looks` trait beside `Holds`) and
build the index's maps inside it. Cost: S. Payoff: the `IndexLookups` tally is structurally
honest and the "index built per entry" mutant is caught by `ExclusionsIndexed` from the same
wrapper. Where: Design, "`IndexLookups` (one per lookup made: paths x (1 + `::` boundaries))".

### 5. UTILITY: the other products in the same file

`check()` (coverage.rs:588-596) loops exclusions x targets to compute `needed`, and
`named_paths`/`is_local` scan every target per re-export. A `first_segment -> packages` map
built once per `check` makes these linear at a few lines' cost, and a growth check with targets
growing at fixed exclusions pins it. Cost: S. Payoff: the "work grows with entries plus
exclusions" claim holds for `check`, not only `compute`. Where: Issue summary, "work is entries x
exclusions" (also exclusions x targets); Task 1 scope.

### 6. UTILITY: dedupe an entry's paths before lookup

`judge_item` (coverage.rs:892-893) pushes the defining path even when it equals `paths[0]`, the
common case, so every lookup is made twice. Deduplicating (`sort`/`dedup` on at most a handful
of `&str`) halves the lookups in the common case and makes the exact pin of item 8 a clean
formula. Cost: S. Where: Design, "for each path, `items.get(path)`...".

### 7. ROBUSTNESS: the self-enforcement gate on the new tests

This repository enforces `raf-workplan` in its own coverage record
(`this_repository_enforces_the_workplan_crates_and_they_are_covered`), so every new unit test in
`coverage/tests.rs` (R3 tests, the differential test, the growth checks) that is not bound in
`docs/capabilities/workplan-two-way-coverage.md` `bindings.tests` is a `coverage-test-unbound`
failure of `make check`; the three renamed/removed product tests must leave the bindings; then
`make docs-sync` regenerates the verification matrix; and the assessments whose inputs digest
`raf_workplan::coverage` (`work/assessments/workplan-two-way-coverage-verification.md`, and the
contract/constraints ones if they bind the symbol) go stale and are re-accepted. The plan says
"bindings" and "re-assess"; make the sequence and the gate explicit so Task 1 cannot be called
done with the self-test red. Also: the repository keeps `CHANGELOG.md` (per-crate bullets under
`[Unreleased]`); the entry belongs there under `raf-workplan`, stated as behaviour (work bound),
not as process. Cost: S. Where: Task 1, "update `docs/capabilities/workplan-two-way-coverage.md`
(Verification text and bindings ...) and the changelog if the repository keeps one".

### 8. ROBUSTNESS: pin the linear quantity exactly where the fixture allows, and decide the empty-record case

The fixture's shapes are known (`big::m{m}::f{i}`: 3 segments, exported and defining path
equal; `big::test_crate::it::t{m}_{i}`: 4 segments; the module item `big::m{m}`: 2), so
`IndexLookups` can be pinned to a formula per entry kind, not only bounded, and asserted
**equal** across 1x/2x/3x exclusions at fixed entries (the plan states this in R1; make it an
assertion, not a sentence), with `ExclusionsIndexed == exclusions` exactly. Then decide the
empty record: `without_exclusions_the_entries_are_judged_and_nothing_is_tested` and the
capability text "with no exclusion nothing is tested" stay true only if an empty index
short-circuits (no lookups when `exclusions.is_empty()`); otherwise the test and the sentence
must change. Recommend the short-circuit. Cost: S. Where: R1 and R2 rows.

### 9. ROBUSTNESS: the differential test needs a generator and adversarial shapes

`raf-workplan` has no PRNG; `Draw` (xorshift64*, splitmix seeding) lives in
`crates/raf-source/tests/support/random.rs` and is not shared. Either copy the six lines into
the coverage test module or lift `Draw` to a shared test-support location. Beyond random draws,
enumerate the shapes that distinguish the index from the scan: sibling shared text (`a::b` vs
`a::bc`), a `Prefix` equal to the whole path (no match), an `Item` on a module that also has a
`Prefix` (both marked, lower index reports), duplicate exclusions, a match through the defining
path only, and the same exclusion set in reversed record order (identical `matched`, mirrored
reasons). Add one determinism check on the index itself: the reason and the `matched` vector
must come from the record order, never from map iteration order (collect hits, take the minimum
index; never `for (k, v) in map` on the result path). Cost: S-M. Where: R3 row, "a differential
test: the indexed result equals a reference product implementation on seeded random
exclusions/paths".

## Not raised

The mutant list, the "first in record order gives the reason" semantics, the choice of hashed
prefixes over a trie (same bound; a trie only wins once glob patterns arrive, see item 1), and
the `KINDS`/`ALL` update in `work.rs` are already in the plan or trivially implied.

VERDICT: NEEDS-REVISION
