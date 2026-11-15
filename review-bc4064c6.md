# Delivery review of bc4064c6 (delta dd382c44..bc4064c6)

## Scope

The delta is one commit that changes 10 files. The rest of the branch passed review at
dd382c44 and was not re-reviewed.

- `crates/raf-workplan/src/obligation.rs`: `Needed::contract_text` drops the
  strict-only `CurrentReview { strict: true, .. }` arm. Every `CurrentReview` now
  returns one string: "a completed current review of the required kind recorded for
  the capability, with a clean verdict when the capability is strict".
- `crates/raf-workplan/tests/obligations.rs`: the shared `evidence` string, used by
  the review-missing and changed-input review-stale cases, and the literal in the
  review-strict-findings case now use that string.
- `docs/capabilities/workplan-reconcile.md`: the evidence cells of the
  `review-missing` and `review-stale` rows in the Usage section now use the same
  string.
- `work/assessments/workplan-reconcile-*.md` (7 files). The Usage assessment has a
  new section digest because the table sits in Usage at lines 210-240. The other six
  have a new `raf_workplan::obligation` module digest. Only digests changed.

## Evidence

- `flock /tmp/raf-build.lock cargo test --locked -p raf-workplan --no-fail-fast`
  exited 0 (the tool's own status, no pipe). Log:
  `review3-wp.log`. Across 39 test binaries and doctests: 800 passed, 0 failed,
  0 ignored, 0 filtered. The tests that cover this change ran and passed:
  `obligation::tests::the_evidence_strings_are_the_evidence_column_of_the_contract`,
  the obligations cases (review-missing, review-stale, review-strict-findings),
  `needed_is_a_typed_value_whose_text_starts_with_the_contract_string`,
  `a_strict_capability_accepts_only_a_clean_current_review` and
  `a_strict_capability_refuses_a_clean_review_of_changed_source`.
- `flock /tmp/raf-build.lock cargo test --locked -p cargo-workplan` exited 0. Log:
  `review3-cwp.log`. Across 23 binaries: 180 passed, 0 failed, 0 filtered.
- `flock /tmp/raf-build.lock cargo test --locked --test self_workplan` exited 0.
  Log: `review3-selfwp.log`. 1 passed. This is the gate binary that requires every
  assessment to be current and accepted, so the renewed digests match HEAD.
- `prose-check docs/capabilities/workplan-reconcile.md` exited 0 with no output.
- I searched the added lines for tracker numbers, issue or PR references, and
  "later", "follow-up" or "out of scope" wording. There were no matches (grep
  exit 1).

## Checks against the brief

1. **Non-strict text.** It changed: the clause ", with a clean verdict when the
   capability is strict" is now appended. The behaviour is unchanged.
   `review_detail` and the strict decision are untouched, and `strict` is still read
   to build the detail ("has the verdict findings"). The change is needed because
   the contract table has one evidence cell per kind, and
   `the_evidence_strings_are_the_evidence_column_of_the_contract` compares that cell
   with `contract_text()`. One string for every `CurrentReview` is the simplest way
   to make the table and the code agree for both strict states. `git grep` across
   tracked files finds the old non-strict literal in no other place. The only
   consumers are `Display` for `Needed` (reconcile output) and `strict.rs`, which
   puts `owed.evidence.to_string()` in the Finding message. Neither matches on the
   text. The strict case now states the clean-verdict condition for both rows. For
   review-missing the condition is real: a findings review recorded against a strict
   capability turns into review-stale.
2. **Agreement.** The two docs cells and `contract_text` match byte for byte, and the
   contract-column test enforces this. The obligations test expectations match the
   code, and all three review cases passed.
3. **Tests.** Both runs exited 0, as shown above.
4. **Assessments.** The reconcile assessments have no body prose; each one holds
   input digests and `verdict: accepted`. I read the reconcile sections against the
   change. Only the Usage table changed, and its new cells are accurate. The Purpose,
   Contract, Constraints, Examples, Explanation and Verification text says nothing
   about the review evidence wording that this delta makes wrong. No renewed
   assessment is inaccurate.
5. **Wording.** No tracker numbers, process wording or deferral wording appears in
   the added lines.

## Findings

- **OPTIONAL.** In `crates/raf-workplan/tests/kinds.rs:418`,
  `a_strict_capability_accepts_only_a_clean_current_review` asserts
  `message.contains("clean verdict")`. That phrase is now in the evidence text for
  every capability, strict or not, so this half of the check no longer tells strict
  from non-strict. The other half, `contains("has the verdict findings")`, still
  does, and the plain-capability case still checks that no finding is raised.
  Protection is unchanged; that half of the check is now redundant.
- **OPTIONAL.** In `crates/raf-workplan/tests/obligations.rs:222`, the
  review-strict-findings case repeats the evidence string as a literal instead of
  using `format!` with the shared `evidence` binding as the other two cases do.

No REQUIRED findings.

VERDICT: PASS
