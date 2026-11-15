<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics Inc. -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Delivery review: strict review mode, range 9824b56d..dd382c44

## Scope

This review covers the whole branch as it will merge: 0227147e (the feature) and dd382c44 (the fix for the earlier BLOCKED unit test). It is read-only. I checked the work against the verbatim issue (`issue.md`), `plan.md` and its `## User rulings` (build order only; no scope rulings), and AGENTS.md sections 7 and 8.

Code paths read:

- `crates/raf-okf/src/profile/{schema,mod,typed}.rs`: the closed key `review_strict` (Flag), placed after `review_required` and defaulting to false.
- `crates/raf-workplan/src/review.rs`: `ReviewStatus.verdict`, `is_clean_current`, `required_reviews` and `review_status`.
- `crates/raf-workplan/src/obligation.rs`: `Needed::CurrentReview { strict }`, `contract_text` and `review_detail`.
- `crates/raf-workplan/src/strict.rs`: the complete-check finding.
- `crates/raf-workplan/src/obligation/sources.rs`: the reconcile and complete obligation caller.
- `crates/cargo-workplan/src/review.rs` and `result.rs`: the CLI `review status` consumer.

## Evidence

Both commands ran from the worktree under `flock /tmp/raf-build.lock`. Each exit status was written straight to its own file, with no pipe in between.

- `cargo test --locked -p raf-workplan --no-fail-fast` gave exit 0 (`review2-wp.exit`, log `review2-wp.log`): 39 test binaries, 800 passed and 0 failed. This includes:
  - `obligation::tests::the_evidence_strings_are_the_evidence_column_of_the_contract` (ok; this was the earlier failure)
  - `a_strict_capability_accepts_only_a_clean_current_review` (ok)
  - `a_strict_capability_refuses_a_clean_review_of_changed_source` (ok)
  - `every_kind_is_produced_by_a_named_fixture_and_every_produced_kind_is_a_kind` (ok; it carries the new `review-strict-findings` obligation case)
- `cargo test --locked -p cargo-workplan` gave exit 0 (`review2-cwp.exit`, log `review2-cwp.log`): 22 test binaries, 180 passed and 0 failed.
- `prose-check` on the five changed docs and on the branch commit messages gave exit 0.
- Not run by me: the raf-okf tests, the hook binaries (`repo_gates`, `docs_contract`, `self_workplan`, `self_census`) and clippy. The hook runs the hook binaries on commit. The raf-okf profile and schema test edits were checked only by reading them.

### Acceptance

1. Under strict review, findings fail and clean passes, side by side. `required_reviews` uses `status.is_clean_current()` when `capability.review_strict` is set. In `kinds.rs::a_strict_capability_accepts_only_a_clean_current_review`:
   - strict with a clean review gives no review finding;
   - strict with a findings review at the same digests gives exactly `["review-stale"]`, and the message contains "has the verdict findings" and "clean verdict";
   - shadowing holds in both directions (an early clean then a late findings review is refused; an early findings then a late clean review is accepted).

   The `obligations.rs` case `review-strict-findings` drives the reconcile and complete path. Its broken repository holds the findings review and its fixed repository holds a clean review at current digests. It asserts the exact obligation line. Both production entry points (the strict check and the obligation sources) reach the new code. **Met.**
2. A clean review with stale digests does not satisfy the requirement. `a_strict_capability_refuses_a_clean_review_of_changed_source` first shows the clean review accepted, then edits `run()`. The result is `["review-stale"]`, the message says "has changed inputs", and the message does not say "verdict findings". **Met.**
3. Without the key, behaviour is as before. The non-strict branch is still `status.is_current()`. The obligation kind choice is unchanged, the non-strict `contract_text` is byte-identical, and `review_detail` with `strict == false` produces the same text as before, for both "no completed" and "has changed inputs: ...". The test asserts that a plain capability with a findings review gives no review finding. The existing non-strict obligation cases pass unchanged. **Met.**

### Docs and assessments

- `review-stale` row: the table's evidence column is the non-strict text. The unit test builds `evidence_of(ReviewStale)` with `strict: false`, so the table and the code agree and the test passes. The description column now names the strict findings condition.
- workplan-code-reviews Contract: the new paragraph matches the code. The status is the latest completed review of the scope and kind; blocked reviews are filtered out; the detail names findings, changed inputs, or both; a capability without the key accepts any verdict at current digests. The renewed digests cover the changed section and `raf_workplan::review`.
- workplan-strict Contract and Examples: the text matches the code and the tests. The tested-by markers name the two new tests, which exist and pass.
- workplan-reconcile: the Usage section holds the table, and its assessment section digest was renewed. Purpose, Contract, Constraints, Examples, Explanation and Verification changed only in the `raf_workplan::obligation` symbol digests. Their prose about missing or stale required reviews stays accurate.
- okf-profile (architecture and capability): the key order and semantics match `schema.rs`, including "no effect without `review_required`".

### Repository rules

- No capability `state:` line changed, and the INVENTORY in `tests/docs_contract.rs` is untouched.
- `work/changes/` is the same as at the base; no record was left behind.
- Added lines contain no tracker numbers, process wording or deferral wording. The only grep hits are the variable name `plan` and the domain word "later" in the shadowing rule.
- Commit subjects are conventional and carry no attribution.

## Findings

- **OPTIONAL**: the workplan-reconcile line "The typed evidence descriptions agree with this closed obligation table" is backed only by the non-strict variant. The strict `contract_text` ("... of the required kind with a clean verdict recorded for the capability") appears in no table row. The row is still true, because a clean current review is a completed current review, and the strict wording is pinned exactly by the `review-strict-findings` obligation case. The sentence and its unit test could cover both variants. Not needed for the contract.
- **OPTIONAL**: the `strict.rs` fix text now adds "; a capability that states `review_strict` needs a clean verdict" to review findings on non-strict capabilities as well. Acceptance is unchanged, but the guidance on non-strict findings now mentions a key they do not use. This is cosmetic.

No REQUIRED findings.

VERDICT: PASS
