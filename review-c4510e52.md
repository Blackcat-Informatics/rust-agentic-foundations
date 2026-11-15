# Delivery re-review: c4510e52 (owner scan moved behind the ratcheted-hit condition)

## Scope

- **Head reviewed:** `c4510e52951a7b1f694c221eaf6de839b3897ad3`, the commit
  `perf(census): scan a COPY's owners only when a ratcheted hit could be its sole cause`,
  on top of `61d896ad`.
- **Fix range:** `61d896ad..c4510e52` (6 files, +14/-10). I read all of it.
- **Merging range:** `21e8c277..c4510e52`. The earlier work keeps the conclusions of
  `review-32effd7b.md`, `review-53f5425f.md` and `review-61d896ad.md`.
- **Out of scope:** step 2 is still a separate landing under the plan.

## Evidence

- **My test run.** At `c4510e52` (I checked `git rev-parse HEAD`), I ran
  `flock /tmp/raf-build.lock cargo test --locked -p raf-census --test census_gate` from
  the worktree. It exited 0 and ran 84 tests, including
  `ratcheted_patterns_record_counted_sites_and_hold_both_directions` with its
  `ratcheted_hit_does_not_hide_a_forbidden_constant` step. The log is
  `/home/paudley/Active/rust-agentic-foundations/.stage/209-reviews/recheck3-census-gate-all.log`.
- **Hook log.** `.worktrees/209-census-debt-a-counted-per-file-ratchet/.stage/209-proof/perf-commit.log`
  shows repo_gates 29, docs_contract 81, self_census 4 and self_workplan 1, all passing.
  The commit `c4510e52` was created and the log records `EXIT=0`.
- **Prose.** `prose-check --stdin` on the commit message exited 0.
- **Assessments bound to the changed symbol.** `grep` over `work/assessments/` finds
  exactly five assessments that bind `raf_census::gate::evaluate`. They are the five
  renewed here, and self_workplan accepts them.

## 1. Boolean equivalence in `report_copies`

The previous code was:

`let other_cause = N || S; if E && !R && (!H || other_cause)`

The new code is:

`let reported = E && !R && (!H || N || S); if reported`

The symbols are:

- `E` = `row.enforced`
- `R` = `rule_copy`
- `H` = `ratcheted_hit`
- `N` = `hits.names.contains(&key)`
- `S` = `owners.values().any(|unit| owners::key(unit) == key && job.matches(unit))`

**Same result for every input.** `!H || (N || S)` and `!H || N || S` are the same
expression, because `||` is associative and both operands are plain `bool`s.

**Nothing else changes.** `N` and `S` are pure reads: no mutation, no early return and no
side effect. Both are evaluated over the same `key`, `job`, `hits` and `owners` bindings as
before. So evaluation order affects only the cost, never the outcome or any state. The
finding text and its position in `findings` are unchanged.

**When the scan runs.** Under `&&` and `||` short-circuiting, `S` is now evaluated only
when `E` holds, `R` is false, `H` holds and `N` is false. The scan therefore runs only when
a ratcheted hit exists in an enforced job, no non-ratcheted rule has already reported the
unit, and no context name explains it. Before this change, `N` and `S` ran for every COPY
match of every job.

## 2. The five renewed census-ledger assessments

Each diff changes one line: the `raf_census::gate::evaluate` input digest. I checked the
layout of every file:

- line 1 is `---`;
- the two `# SPDX-...` lines follow (FileCopyrightText, then License-Identifier
  CC-BY-4.0);
- then `type: Assessment` and the matching `id: assessment:census-ledger-<section>`;
- each file ends with `verdict: accepted`, `by: agent`, `---`.

This is the same form as an untouched assessment (`census-patterns-contract.md`). The diff
shows no layout change, so the committed form was kept.

I read each section of `docs/capabilities/census-ledger.md` against the change:

| Section | Judgment |
|---|---|
| Purpose | Names homes, forbidden code and justified distinct implementations, tied to the current workspace. It does not depend on when the COPY check computes its causes. Accurate. |
| Examples | Ledger and provider example cases. The behaviour is unchanged, as shown in part 1. Accurate. |
| Constraints | Adapter-proof limits, locked-writer acquisition boundary, unavailable execution refusal, explicit locked writer. None concerns `report_copies`. Accurate. |
| Explanation | One native RDF authority, physical ownership, exact sanctions, two-sided staleness, "reports every selected physical item once". The change does not alter any of these. Accurate. |
| Verification | Configured debt example and provider/native-state/declaration fixtures. Unaffected. Accurate. |

The renewals were required by the digest change, which self_workplan would otherwise
refuse. They were warranted, and every section's text is still accurate.

## Findings

No REQUIRED findings.

- **OPTIONAL A, still open in part.** When the scan does run, it remains a linear pass
  over `owners.values()` that builds two `String`s per unit through `owners::key`. A
  lookup by position (`owners.get(&(RelPath, start_byte))`), or a field comparison that
  allocates nothing, would remove that cost. Since this commit, the scan is limited to
  units whose COPY a ratcheted hit could explain alone, which makes the cost minor.
- **OPTIONAL B, unchanged from the previous review.** The `census-patterns.md` Ratcheted
  patterns paragraph does not list a forbidden name as another cause that keeps the
  generic COPY finding.

VERDICT: PASS
