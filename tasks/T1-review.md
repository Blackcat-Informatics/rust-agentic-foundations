# T1 review: commit d5793a1

VERDICT: BLOCKED

## Blocking finding

F1. The change-set probe does not catch the regression it documents.
`crates/raf-okf/src/profile/mod.rs:602-603` builds `listed` and then ticks
`ChangeIdsSet` on a separate line; the lookup at `mod.rs:641` carries no tick. The
test `a_change_at_the_cap_judges_each_affects_entry_once_against_one_set`
(`cost_tests.rs:~158-176`) asserts `ChangeIdsSet == count` and `AffectsEntries == count`,
and both stay exact under either regression below. Run in a scratch clone of d5793a1:

* `listed` deleted, `listed.contains(capability)` replaced by
  `changes.iter().any(|c| c == capability)`, the `add(ChangeIdsSet, ..)` line left in
  place: all 6 cost tests pass, no ceiling hit (quadratic scan over 20k entries).
* The `listed` build moved into the `affects` loop, the tick left in place: all 6 cost
  tests pass, after 194.58 s (it only finishes by running the quadratic work).
* Moving or deleting the tick together with the build fails the exact assertion, but that
  is a mutant of the counter, not of the bound.

So the okf-profile Verification text "the set of `changes` is built once, not once
per entry" (`docs/capabilities/okf-profile.md`, Cost fixtures bullet) is not enforced.
Fix: tick where the work is done (for example `.inspect` on the set build, and a counted
lookup through a small helper or a `ChangeLookups` tick at `mod.rs:641`), with a ceiling,
so a per-entry rebuild or linear scan passes `count` on the second entry.
Alternatively state in the Verification text only what is checked.

## Findings by item

(a) Vacuity. Every counted test asserts an exact value or the growth idiom; the growth
tests (`assert_linear`, `cost_tests.rs`) assert `one > 0`. The exact tests tie the tally to
`count`/`blocks`/`lines` (each asserted > 20000, 10000, 5000), so a zero tally fails them.
No test passes with a zero tally. Chokepoints: `scan` (`body/blocks.rs:80-130`) is the only
caller of `opens_fence` and `closes_fence`, so the two ticks cover the whole fence scan;
`.inspect` in the `by_name` build (`records.rs:~593`) ticks once per collected value;
`visible_content` is the only reader of `generated_blocks` in the profile. The one hole is F1.
Minor: with `let first = 0` alone, the growth test fails on a slice-range panic
(`body_rules.rs:59`), not on its growth assertion; a correct-output variant (skip blocks
before the section) fails on the ceiling as intended.

(b) Mutants rerun in a scratch clone (/tmp/t1review-*): `let first = 0` fails
`a_capability_at_the_cap_looks_at_each_block_once` (ceiling 10000: 10001) and the growth
test; `by_name` inside the limits loop fails the evidence test (MeasuresIndexed ceiling
44605: 44606), fast; the deleted `listed` index fails only if its tick is removed with it
(cost_tests.rs:168) and passes if the tick stays (F1).

(c) cfg(test) only: pass. `work.rs` `add` is `#[cfg(not(test))] #[inline] fn add(_, _) {}`;
the Tally/ceiling machinery is `cfg(test)`. The diff to non-test code is imports, `mod work;`
(private), one `add` call per site and `.inspect(|_| add(..))`; no signature or visibility
changes. `cargo build -p raf-okf --release --locked` is clean.

(d) `validate_cost.rs` has no clock (no Instant/Duration/elapsed); it still checks that the
cap-sized documents parse and validate with exact entry counts, and that the capability is
refused for `SectionDuplicated`. Meaningful as a functional check.

(e) Makefile/CI/AGENTS.md/CONTRIBUTING.md: only the raf-okf clause removed; the raf-source
reason is kept. `make -n test-release` prints only `cargo test -p raf-source --release --locked`.
Text is true. Nit: `AGENTS.md:159-160` now has a reflowed line longer than its neighbours
(the "`make fuzz` checks..." sentence starts mid-line); cosmetic.

(f) Bindings resolve: the three renamed `validate_cost` tests and the six `cost_tests`
tests exist with those names. Verification spot checks: "a table built for each limit would
pass the count at the second limit" is true (confirmed by the mutant); "set of changes built
once, not once per entry" is not enforced (F1).

(g) Repo rules: no clock words in the added files (the grep hit is "swallows"), no issue/PR
numbers or process wording found in the added text, SPDX headers present on `work.rs` and
`cost_tests.rs`. Fuzz seeds `fuzz/corpus/parse-document/seed_full_*` are frozen snapshots
used as fuzz inputs (the `source-index` one already differs from its document by about 1800
diff lines; the AGENTS one by about 93 lines before this commit); not a maintained reference.
No change needed.

(h) `cargo fmt --all --check` clean; `cargo clippy -p raf-okf --locked --all-targets -- -D
warnings` clean; `cargo test -p raf-okf --locked` all green (20 lib tests including the
6 cost tests and the `work::tests`).

Note: the worktree currently has uncommitted raf-workplan edits (later tasks); they are not
part of d5793a1 and were not touched.
