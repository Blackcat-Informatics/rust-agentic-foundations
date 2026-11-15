# Review of e06b53e6 (measured evidence requirement)

## Scope

- Worktree `.worktrees/267-workplan-let-a-capability-require`, range `f7b87db8..e06b53e6` (one commit, 40 files).
- Read: the verbatim issue (`.stage/workplan-let-a-capability-require/issue.md`), `plan.md` with its `## User rulings`, AGENTS.md, the rust-work skill, the whole source and test diff, the changed capability and architecture prose, and the prose of every section whose assessment was renewed.
- Not run: workspace gates, clippy, fmt, the hook binaries, mutation testing or timing checks.

## Evidence

- `flock /tmp/raf-build.lock cargo test --locked -p raf-okf -p raf-workplan --no-fail-fast` ran at the head after the raf-okf helper merge. Log: `.stage/workplan-let-a-capability-require/review-tests.log`. Cargo's own exit status went to `review-tests.status` and reads `EXIT=0`. The log has 62 `test result:` lines: 1082 passed, 0 failed, 0 ignored, and doctests are included. Every new test is listed as `ok`:
  - `a_capability_requiring_measured_evidence_is_refused_without_it_and_accepted_with_it`
  - `a_capability_that_does_not_require_measured_evidence_is_judged_as_before`
  - both `a_record_that_states_units_or_conditions_states_them_for_every_value` tests (raf-okf and raf-workplan)
  - `completion_waits_for_the_measured_evidence_a_capability_requires`
  - `every_kind_is_produced_by_a_named_fixture_and_every_produced_kind_is_a_kind`
  - `a_built_evidence_record_renders_to_a_record_that_reads_back_equal`
- `prose-check` on `docs/architecture/okf-profile.md`, `docs/capabilities/okf-profile.md`, `docs/capabilities/workplan-evidence-claims.md`, `strict.rs`, `typed.rs` and `violation.rs` exited 0.
- `git diff --name-status` shows no added or deleted files. `tests/docs_contract.rs` (INVENTORY) is not in the diff, and no `work/changes/*` file is in the diff. At the head, the `state:` of `okf-profile`, `workplan-evidence-claims` and `workplan-strict` is `delivered`, the same as at the base. `complete.log` records `cargo workplan change complete` removing `work/changes/measured-evidence.md`.
- A grep of the added lines for tracker numbers, process wording and deferral wording found nothing. The only matches were identifiers such as `plan.evidence()`. The commit subject has no tracker number.

## Acceptance check

1. **Refused without a measured record, accepted with one.** This is met. `strict::check` (`crates/raf-workplan/src/strict.rs`, called unconditionally from `Workplan::check` at `model.rs:424`) raises `measured-evidence-missing` at the key `measured_evidence_required`. It does so unless some loaded evidence record lists the capability in `for` and `EvidenceRecord::states_measurement()` is true. That requires at least one value, `conditions` present, and a non-empty unit for every value.
   - `check`: the strict test asserts exactly `["measured-evidence-missing"]` at the capability path and key in three cases: with no record, with a values-only record, and with a conditions-only record. With the complete record it asserts `[]`.
   - `change complete`: `completion_waits_for_the_measured_evidence_a_capability_requires` asserts that completion is refused (`AfterRemoval` condition, record still on disk) for no record and for a values-only record, and that it completes with the measured neighbour.
2. **A measured record missing a unit or conditions is refused beside a complete one.** This is met under the delivered definition of "measured": a record that states `units` or `conditions`.
   - raf-okf `records.rs` uses `assert_only` to check four refusals beside the clean `measured()` fixture: a value without its unit gives `ValueWithoutUnit`, conditions with no units give `ValueWithoutUnit`, units with no conditions give `ConditionsMissing`, and an empty unit gives `EmptyText`.
   - The workplan `cases.rs` rule cases map both new rules to `okf-value-without-unit` and `okf-conditions-missing`.
3. **A capability without the key is judged as before.** This is met. The flag defaults to `false`. The strict test runs a values-only record without the key and with `measured_evidence_required: false`, and both give `[]`.

**The "only when units or conditions are stated" design.** A record with values and no units or conditions loads clean. It does not satisfy the capability requirement: `states_measurement` is false, and both the strict test and the completion test assert the refusal for it.

This matches the issue's wording ("refused when the record declares itself measured") and its compatibility clause ("existing documents and records are unaffected"). It differs from the plan's parenthetical "A measured record (declares values)". Under that reading every existing values-only record would be refused, which contradicts the issue's compatibility clause. The issue governs, so this is not a finding. The stage plan text could be aligned.

## Assessment sections read against the change

- **okf-profile Purpose, Contract, Usage, Examples, Constraints, Explanation, Verification:** all accurate.
  - The new Contract `### Measured evidence` clause matches `check_between`, `states_measurement`, the flag parse and the writer order.
  - "Keys are exactly those of the profile" still holds, because both new keys are in the closed tables.
  - The Positioned-results list and the Verification fixture list are not exhaustive and contain nothing that is now false.
  - The Contract wording "They optionally state `measured_evidence_required: true`" is narrower than the code, which also accepts `false`. It is not inaccurate.
- **workplan-evidence-claims Contract:** accurate in substance. It states the finding, its key, the satisfying record, completion, the opt-out, "not debt" (the kind is not in `DEBT_KINDS` and is pushed after `debt::judge`), and "does not judge a verdict or a limit". One edge-case imprecision is listed as an optional finding below.
- **workplan-strict Purpose, Contract, Constraints, Examples, Explanation, Verification:** accurate. None of these sections claims an exhaustive list of capability-level requirements. "The kinds" uses "includes". The sentence "Capabilities without required kinds retain the existing complete policy" refers to review kinds. The renewal was triggered by the `raf_workplan::strict` symbol digest, and no sentence is now false.

No renewed assessment covers a section that is inaccurate for the delivered behaviour.

## Schema closure and writer round trip

- `CAPABILITY` gains `optional("measured_evidence_required", Flag)`, and `EVIDENCE` gains `optional("units", Units)` and `optional("conditions", Conditions)`. The tables stay closed: `an_unknown_or_foreign_key_is_reported_on_every_type` was updated to treat the new keys as foreign on every other type, and the schema order test pins the order.
- The flag accepts only `true`/`false`. `yes` and `[true]` are refused beside the valid neighbours. `units` must be a map with non-empty texts, and `conditions` must be non-empty text.
- `EvidenceRecord::to_document` writes `units`, then `conditions`, after `limits`. `a_built_evidence_record_renders_to_a_record_that_reads_back_equal` round-trips a record with three units and conditions to an equal value. `empty_lists_and_absent_keys_are_written_as_the_profile_reads_them` asserts that neither key is written when absent, and that record reads back equal.
- Capabilities have no typed writer. They round-trip through the generic `Document` codec, which is unchanged.

## Findings

All findings are optional. None blocks the issue contract.

1. **OPTIONAL: a unit that names no value is accepted silently.**
   - The code: `check_between` checks values against units but not units against values. `units: {peak: MiB, stray: s}` beside `values: {peak: 41}` loads clean, and so does a record with `units` and no `values` at all.
   - The precedent: the neighbouring rule `LimitWithoutValue` refuses the mirror case.
   - The issue: it describes `units` as "a unit for each measured name".
   - The impact: the realistic typo (a misspelt unit name) is still caught, because the real value then has no unit. Refusing an orphan unit, with the same failing-verdict exemption that limits use, would match the repository's closed, fail-hard schema policy.
2. **OPTIONAL: a failing record satisfies the requirement.** A record with `verdict: fail` that states values, units and conditions satisfies `measured_evidence_required`. The Contract says so explicitly ("does not judge a verdict or a limit"), and the issue does not ask for more. Claims already treat a failing record as unsupported (`ClaimProblem::Fails`), so this is worth a deliberate decision. It is not a defect against this contract.
3. **OPTIONAL: an inaccurate test comment.** In `crates/raf-workplan/tests/strict.rs` (`a_capability_requiring_measured_evidence_is_refused_without_it_and_accepted_with_it`), the comment says "Values alone, a record of another subject's values, and units without conditions state no measurement". The cases are actually values alone and conditions without values. Neither "another subject's values" nor "units without conditions" is exercised there. The units-without-conditions case would fail at load with a different kind.
4. **OPTIONAL: overlapping test.** The raf-workplan `a_record_that_states_units_or_conditions_states_them_for_every_value` repeats the raf-okf test of the same name. Its kind mapping is already covered by the `cases.rs` rule cases through `every_kind_is_produced_by_a_named_fixture...`. Its last assertion repeats the previous test's. AGENTS.md section 6 asks for overlapping tests to be removed. The workplan-evidence-claims marker could cite the raf-okf test instead; workplan-strict already binds raf-okf tests across crates.
5. **OPTIONAL: an imprecise edge case in workplan-evidence-claims.** The Contract says "a record that states either gives every value its unit and states its conditions". A record with `units` and no `values` is accepted without `conditions`. The okf-profile wording ("a record that states units and holds a value also states its conditions") is the precise form.
6. **OPTIONAL: the documented rule and the code that enforces it are bound to different capabilities.** The finding is raised in `raf_workplan::strict`, which workplan-strict binds. Its contract is written in workplan-evidence-claims, which binds none of that code, so a later edit to the rule would stale workplan-strict's assessments, not the section that describes it. Precedent supports either fix: `review_required`, the issue's named model, is documented in the workplan-strict Contract. One option is a sentence there; the other is binding the enforcing symbol from workplan-evidence-claims.

VERDICT: PASS
