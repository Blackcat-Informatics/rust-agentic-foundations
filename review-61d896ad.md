# Delivery re-review: 61d896ad (fixes for two bot review findings)

## Scope

- **Head reviewed.** `61d896adc4f40c866f3043887278c163bf4c8224`, the commit
  `fix(census): keep hard COPY findings beside a ratcheted hit and refuse ratcheting through an unenforced fingerprint`,
  on top of `53f5425f`.
- **Ranges.** The fix range is `53f5425f..61d896ad` (31 files, +137/-51); I read all of
  it. The branch as it will merge is `21e8c277..61d896ad`. The earlier work keeps the
  conclusions of `review-32effd7b.md` and `review-53f5425f.md`. I re-judged only the work
  this commit changed.
- **Code read beyond the diff.**
  - `matching_units` and `report_copies` in `gate/evaluate.rs`
  - `Job::matches` in `job.rs`
  - `owners::index` and `owners::key`
  - `gate/subjects.rs` and `gate/rules.rs`
  - `LedgerJob::referenced` in `report/ledger.rs`
  - the ratcheted refusal in `report/ledger/declarations.rs`
  - the new test steps in `crates/raf-census/tests/census_gate.rs`
- **Step 2** (job-level `ratchetedRule`, `RegisteredDebtWrite`) is still a separate landing
  under the plan. It is not judged here.

## Evidence

My own runs at `61d896ad` (`git rev-parse HEAD` checked), each under
`flock /tmp/raf-build.lock` from the worktree. Logs are in
`/home/paudley/Active/rust-agentic-foundations/.stage/209-reviews/`:

- `cargo test --locked -p raf-census --test census_gate -- ratchet ledger_pattern_declarations`
  exited 0 (`recheck2-census-gate.log`). It ran 2 tests:
  `ratcheted_patterns_record_counted_sites_and_hold_both_directions`, which now includes
  `ratcheted_hit_does_not_hide_a_forbidden_constant`, and
  `ledger_pattern_declarations_are_closed_referenced_and_not_executable_by_presence`, which
  now includes the second-job fingerprint case.
- `cargo test --locked -p raf-census --test census_gate` exited 0
  (`recheck2-census-gate-all.log`). It ran 84 tests, the whole binary, because
  `report_copies` and every caller of `referenced()` decide the COPY outcome of every
  existing gate fixture.
- `cargo test --locked -p cargo-census --test delivery -- debt_command_adopts_refuses_growth`
  exited 0 (`recheck2-delivery.log`). It ran 1 test.
- **Hook.** The log at
  `.worktrees/209-census-debt-a-counted-per-file-ratchet/.stage/209-proof/bot-commit.log`
  shows repo_gates 29, docs_contract 81, self_census 4 and self_workplan 1, all passing,
  and commit `61d896ad` created.
- **Prose and wording.** `prose-check` on `census-ledger.md`, `census-patterns.md` and the
  commit message exited 0. A grep of the added lines in `21e8c277..61d896ad` found no
  tracker numbers or process or deferral wording. The only hits were the identifier
  `both`.
- **State, inventory and change records.** `git diff 21e8c277..61d896ad` shows 0
  `state:` changes in `docs/capabilities` and no change to `tests/docs_contract.rs` or
  `work/changes/`. `work/changes/` holds only two records that belong to other changes.

## Bot finding 1: a ratcheted hit hid other COPY causes

A unit becomes a COPY in `matching_units` through exactly three branches:

1. `job.matches(unit)`. `Job::matches` covers forbidden body, shim and table fingerprints
   and forbidden constants.
2. `hits.names` (forbidden or context names).
3. A hit of any referenced id: a rule fingerprint or a pattern, with the same home-exempt
   condition.

`report_copies` now handles these as follows:

- Branch 1 and branch 2 are recomputed as `other_cause`:
  `hits.names.contains(&key) || owners.values().any(|unit| owners::key(unit) == key && job.matches(unit))`.
  `owners::index` is built from `census.units()`, the same set `matching_units` filters,
  so every unit-backed COPY is found. A subject-only match (`gate/subjects.rs`) has no
  unit, so `job.matches` cannot apply to it, and `other_cause` is correctly false.
- Branch 3 for a non-ratcheted id still sets `rule_copy` and emits that rule's own
  finding, so the generic finding is not needed. This is unchanged and correct.
- Branch 3 for a ratcheted id now sets `ratcheted_hit` instead of `rule_copy`.
  `sink.divert(id, &key)` is still called before the `continue`, so the hit is recorded as
  a site whether or not the generic finding is emitted.
- The condition `row.enforced && !rule_copy && (!ratcheted_hit || other_cause)` therefore
  suppresses the generic COPY only when the ratcheted pattern is the only cause.

**The new step `ratcheted_hit_does_not_hide_a_forbidden_constant`.**

- The clean neighbour, with the same ledger, a `forbiddenConstant 7` and no 7 in the
  source, stays clean.
- Adding `let _=7;` to `one` after its keyed literal keeps that site's key
  `(one, "alpha")`. The test asserts exactly one finding containing `one is a COPY`, and
  that the record still has 3 sites.
- Without the fix, the ratcheted hit set `rule_copy = true`. The constant has no other
  reporter, so the findings would be empty and `assert_eq!(report.findings().len(), 1)`
  fails. The step would fail without the fix (judged from the code; per the no-mutation
  rule I did not revert the code to run it).

**`census-patterns.md`.** The Ratcheted patterns paragraph now says the suppression
happens "only when the ratcheted pattern alone explains the match", and names forbidden
constants, structural fingerprints and non-ratcheted rules as causes that still fail. This
is accurate. It omits a forbidden name (`hits.names`) as a cause; see OPTIONAL B.

## Bot finding 2: references through a forbidden rule fingerprint

- **Single definition.** `pub(crate) fn referenced` now exists only as
  `LedgerJob::referenced` in `crates/raf-census/src/report/ledger.rs`.
  `rules::referenced` is deleted, along with the `Fingerprint` and `LedgerJob` imports
  that only it used.
- **Callers.** All five callers use `row.referenced()` / `job.referenced()`, and no
  `rules::referenced` remains:
  - `evaluate.rs` `matching_units` and `report_copies`
  - `rules.rs` `requested`
  - `subjects.rs` `append`
  - `declarations.rs`, the ratcheted refusal
- **Behaviour.** The move is a pure refactor for the four gate callers. The refusal now
  matches `job.referenced().any(|id| id == pattern.id)`, which covers declared patterns
  and `rule:` fingerprints.
- **Test.** The new case adds a second job that reaches `approved-call` only through
  `forbiddenFingerprint "rule:approved-call"`:
  - unenforced, it is refused with a message naming `not enforced` and `second`;
  - enforced, the same graph loads (the valid neighbour).

  Before the fix, the unenforced case would have loaded, because `job.patterns` is empty
  for `#second`.
- **Gate consistency.** `report_copies` already iterates the referenced ids, so a pattern
  reached through a rule fingerprint is diverted to the sink in the same way.
- **`census-ledger.md`.** The text now says "by a declared pattern or a forbidden rule
  fingerprint", which matches the code.

## Renewed assessments (23)

I read every renewed assessment against its section and against this commit's change.

| Assessments | Input that changed | Judgment |
|---|---|---|
| census-detectors (7: purpose, contract, usage, examples, constraints, explanation, verification) | `raf_census::gate::subjects` digest (the one-line call-site move) | The renewal was required, because self_workplan fails on a stale digest. The behaviour is identical. The detectors text makes no claim about how referenced ids are collected or about COPY suppression. Accurate. |
| census-rule-api (6: purpose, contract, usage, constraints, explanation, verification) | `raf_census::gate::rules` digest (`referenced` removed, `requested` caller moved) | Required by the digest change; same behaviour. No section describes `referenced` or the COPY finding. The `DebtWrite` sentence from the earlier fix still matches `write_debt_with`. Accurate. |
| census-normal-form (3: purpose, examples, verification) | `raf_census::report` digest (`LedgerJob::referenced` added in `report/ledger.rs`) | Required by the digest change. These sections cover the normal form, fingerprints and report regeneration and are unaffected. Accurate. |
| census-ledger (5: purpose, constraints, examples, explanation, verification) | `gate::evaluate` and `report::ledger` digests | None of the five sections states the old suppression rule or the old reference scope. "Staleness in both directions...", "reports every selected physical item once" and the locked-writer text are unaffected. Accurate. |
| census-ledger contract | section text changed (fingerprint reference) | Matches `declarations.rs`. Accurate. |
| census-patterns contract | section text changed (suppression only when the pattern alone explains the match; fingerprint reference) | Matches `report_copies` and `declarations.rs`. Accurate, apart from the omission in OPTIONAL B. |

No renewal was unwarranted. Each one follows a real input digest or section change that
the self_workplan gate would otherwise refuse, and each section's text is accurate for
the delivered behaviour. The author did not read these before renewing. That is a process
fault on their side, but it leaves no defect in the committed text.

## Findings

No REQUIRED findings.

- **OPTIONAL A: `other_cause` is computed for every COPY match.** It runs even when there
  is no ratcheted hit, scans `owners.values()` linearly, and builds two `String`s per unit
  through `owners::key`. This makes report_copies O(COPY matches × units) with
  allocations, also in repositories that never ratchet.

  Suggested change: compute it only when `ratcheted_hit` is true, and look the unit up by
  position instead of scanning. The match carries `file` and `start_byte`, so either use
  `owners.get(&(RelPath, start_byte))` or compare the unit's fields without building the
  key. The result is the same, so this is not a correctness issue.
- **OPTIONAL B: `census-patterns.md` does not list a forbidden name as another COPY
  cause.** The code keeps the generic finding when `hits.names` contains the unit, but the
  paragraph lists only forbidden constants, structural fingerprints and non-ratcheted
  rules. Add "a forbidden name" for completeness.

VERDICT: PASS
