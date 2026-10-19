# Plan (Rev 2): index the coverage product (issue 29)

## Issue summary
`raf_workplan::coverage` judges every public item and test of a crate against every exclusion
of the record (`exclusion_for`, coverage.rs, called from `Judge::judge`): work is entries x
exclusions, pinned exactly by `coverage::tests::every_exclusion_is_tested_against_every_entry_exactly_once`
and two growth tests. The issue asks for an index so the work grows with entries plus
exclusions, and a linear-growth check in place of the exact product pin. Constraint: every
exclusion that matches an entry is still marked (so unused exclusions are reported) and the
report is unchanged.

## Standing goals / governing decisions
CLAUDE.md: no timing runs (counts only; tell every agent started); no issue/PR numbers or
process wording in committed files; no attribution; no deferrals; no lint weakened;
`make check` before done (cross-check via hosted CI where targets are absent). Prior art: issue 21
pinned the product deliberately and declined an index at that time; PR for 21 is the precedent
(the PR cited in the brief is that change, not an attempt at this issue).

## Design
`matches` has two forms: `ExclusionItem::Item(p)` matches a path equal to `p`;
`ExclusionItem::Prefix(p)` matches a path `p::rest` (never `p` itself, never `a::bc` for `a::b`).
One private type `ExclusionIndex` owns the exclusions slice, `items: HashMap<&str, Vec<usize>>`,
`prefixes: HashMap<&str, Vec<usize>>` and the `matched` flags, built by one constructor (the
indices index both the slice and the flags, so they live together). For an entry with matching
paths P: with no exclusions nothing is looked up; otherwise for each path one `items` lookup and
one `prefixes` lookup for every proper ancestor taken with the existing `ancestors` helper of
coverage.rs (so the boundary rule is the one `Bound` uses), every index found is marked, and the
exclusion returned is the smallest index (record order, as now). `matches` and the slice-walking
`exclusion_for` are deleted from production; the product reference implementation lives only in
the test module as the oracle. Counted work (`work.rs`): `ExclusionTests` is replaced by
`ExclusionsIndexed` (one per exclusion inserted), `IndexLookups` (one per map lookup) and
`ExclusionMarks` (one per index marked), all ticked at the site that does the work (insertion,
lookup, mark); `EntriesJudged` stays. Duplicate exclusions cannot reach judging (the record
loader refuses them); the index keeps both anyway and a test-only hand-built list shows it.
Scope boundary: `check` also loops over exclusions x crate targets to find the packages needed;
that is a product over crates, not over entries, and this change leaves it alone; the capability Verification states it as a fact of the system and not as a scope note; stated in the
capability text next to the judging statement.

## Completeness contract
Ledger R1 (build an index so the work grows with entries plus exclusions) -> plan R1; ledger R2
(replace the exact product pin with a linear-growth check) -> plan R2; ledger R3-R6 (every
matching exclusion returned and marked, report of unused exclusions unchanged, an index that
stops at the first match would hide unused exclusions) -> plan R3 and R4.
| R | Requirement | Task | Executable check |
|---|---|---|---|
| R1 | The work grows with entries plus exclusions | 1 | counted, exact: `ExclusionsIndexed == exclusions`; `IndexLookups ==` the sum over entries of (paths x (1 + proper ancestors)) computed in the test from the fixture; independent of the exclusion count (same entries, 10x the exclusions: equal `IndexLookups`); with no exclusions `IndexLookups == 0` and `ExclusionsIndexed == 0` |
| R2 | The exact product pin is replaced by a linear-growth check | 1 | the four product tests are replaced: (a) one-variable growth checks on TOTAL counted work (`one>0, three+one<=2*two`) for entries at fixed exclusions and exclusions at fixed entries; (b) the discriminating test: scale entries AND exclusions together in units 1, 2, 3 and require `three + one <= 2 * two` on total work (the product's 9 + 1 > 2 * 4 fails; the index passes); (c) a ceiling linear in entries + exclusions on every kind (`measure`) so any per-entry scan of the exclusions panics; (d) `without_exclusions_the_entries_are_judged_and_nothing_is_tested` is kept in meaning: no exclusion, no lookups (renamed only if its text changes, then re-bound in the capability); record in validation.md that (b) and (c) FAIL against the old product code in a scratch clone |
| R3 | Every matching exclusion is returned and marked; unused exclusions reported as before | 1 | via `Workplan::check`: an exclusion matched only by entries whose first match is another exclusion yields no `coverage-exclusion-unmatched` finding; an exclusion matching nothing yields exactly one (existing test); an Item and a Prefix both matching one entry are both marked; the first in record order gives the reason (existing test); an enumerated cross product (every exclusion of a small alphabet of Item/Prefix over paths of depth 1-4, including `a::b` vs `a::bc`, a prefix equal to the whole path, a prefix longer than the path, shared prefixes, an exclusion that is a prefix of another) compares the index with the oracle on BOTH the first match and the full matched-flags vector, and is repeated on a rebuilt index to show the result does not depend on map iteration order; duplicates only on a hand-built list |
| R4 | The report is unchanged | 1 | `crates/raf-workplan/tests/coverage.rs` has an empty diff and passes; every test in it, among them `the_first_matching_exclusion_gives_the_reason` and `an_exclusion_without_a_reason_or_that_matches_nothing_fails_naming_the_file`, and the other coverage report tests that `cargo test -p raf-workplan --locked --lib coverage::` and `--test coverage` run pass unchanged (the tests the capability binds besides the four replaced); the capability Contract text for coverage (lines about exclusions) has an empty diff |
Mutants (scratch clone, each must fail a named test): the index stops at the first match; the whole path used as a prefix key (prefix == path wrongly matches); the `ancestors` walk skipping the nearest ancestor; the largest index chosen instead of the smallest; the index built once per entry (R2b/R2c fail); the marking loop removed (R3 fails); the old product code (R2b, R2c fail).

## Tasks
### Task 1: the index (commit type `perf(raf-workplan): ...`)
Files: `crates/raf-workplan/src/coverage.rs`, `coverage/tests.rs` (module doc rewritten to state
the present behaviour only), `work.rs` (kinds, `KINDS`, `ALL`, tests of the table),
`docs/capabilities/workplan-two-way-coverage.md` (frontmatter bindings AND the generated table
via `make docs-sync`, the Verification text, assessments re-read and accepted), `CHANGELOG.md`
(an entry under Unreleased; `Work` is crate-private so no public API entry). Every new unit test
is bound in the capability or `coverage-test-unbound` fails. Focused validation:
`cargo test -p raf-workplan --locked --lib coverage::`, `cargo test -p raf-workplan --locked --test coverage`,
`cargo test --locked --test docs_contract --test self_workplan`, `make fmt clippy doc deny reuse issue-refs`,
`cargo workplan check`, `assess --list` clean, mutants in a scratch clone. Independent review
(T1-review.md), commit, push, issue update (`stagectl comment`).
### Task 2: handoff
Full `make check` sequence. `make cross-check` cannot complete on this host: the host selects the
single installed dated nightly through `RUSTUP_TOOLCHAIN` (set by the Stage configuration), which has the
`x86_64-pc-windows-gnu` standard library and lacks the Apple and FreeBSD ones, and the machine
rule forbids installing or selecting another toolchain. The recipe's own failure is recorded
verbatim in validation.md. Run by hand: `RUSTFLAGS=-Dwarnings cargo check -p raf-source --locked --all-targets
--target x86_64-pc-windows-gnu` (the target this change cannot affect: `raf-source` is untouched);
the Apple and FreeBSD targets are reported as UNRUN locally, with the hosted `cross-check` job as
the only evidence, and the PR says so. This is a machine limit disclosed, not a skipped step.
PR "Closes #29" with the mutant table and disclosures, plan posted, least-confident note.

## NEEDS-JUDGMENT dispositions
Prior-art items: the file is `crates/raf-workplan/src/coverage.rs`; the PR cited in the brief is
the change that pinned the product, not an attempt; the issue names no ADRs/packages: the plan
identifies the files itself.

## Declined
A trie over prefixes: a hash map by prefix text per `::` boundary is simpler and has the same
bound. Sorting exclusions: needless. A shared index crate: one consumer.

Ledger mapping: the ledger's six rows are R1 (build an index so work grows with entries plus
exclusions), R2 (replace the exact product pin with a linear-growth check), R3-R6 (the
constraint split: marks every matching exclusion, not only the first kept; an index returns every
matching exclusion per entry; the unused-exclusion report stays the same; an index that stops at
the first match would hide unused exclusions). They are covered by the plan's R1, R2, R3 (R3-R6
of the ledger: every one of those four is the R3 test list) and R4. NEEDS-JUDGMENT: "linear-growth
check" means the repository idiom (`one > 0`, `three + one <= 2 * two` on counted work); the index
must reproduce the output of `exclusion_for` exactly (first match in record order gives the
reason, every match is marked), not only the unused-exclusion report.

## Enhancements considered
Shared `PathIndex` with `Bound` (enhancement audit): declined, `Bound::covers` is already a set
lookup per ancestor (not a product) and unifying is a refactor outside the issue; the boundary rule
is shared by reusing `ancestors`. Exclusion effectiveness counts / shadowed-exclusion diagnostic:
declined, a new feature (and the report must stay unchanged, R4). Dedupe of duplicate entry paths: declined, entries repeat per paths by design and the index lookup is per path already counted. `Counted` wrapper: not used, the three ticks sit at the insert, lookup and
mark sites. The exclusions x targets loop in `check`: out of scope, stated above. A PRNG: none;
the comparison is an enumerated cross product.
