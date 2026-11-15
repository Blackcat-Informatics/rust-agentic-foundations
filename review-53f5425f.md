# Delivery re-review: 53f5425f (counted site ratchet for patterns, step 1 of 2)

## Scope

- New head: `53f5425f22c9408b8e5e27620d6a92095a760584`. It is the fix commit
  `fix(census): frame debt site keys with purrdf, count site rows, and guard a burned-down ratchet`
  on top of `32effd7b`.
- Reviewed the fix range `32effd7b..53f5425f` (29 files, +173/-106) in full. Also reviewed
  the merging range `21e8c277..53f5425f`, judging the changed work and the open findings
  from `review-32effd7b.md`. Unchanged parts of `32effd7b` keep that review's conclusions.
- Step 2 (job-level `ratchetedRule`, `RegisteredDebtWrite`) is still a separate landing
  under the plan. It is not judged here.

## Evidence

My own runs at `53f5425f` (`git rev-parse HEAD` checked), under `flock /tmp/raf-build.lock`
from the worktree. Logs are in `/home/paudley/Active/rust-agentic-foundations/.stage/209-reviews/`:

- `cargo test --locked -p raf-census --test census_gate --test debt --test reports -- ratchet ledger_pattern_declarations ledger_core_nodes_refuse_unknown canonical_reports_preserve_golden counted_nodes both_native invalid_groups`
  exited 0 (`recheck-raf-census.log`). It ran 8 tests:
  - `ratcheted_patterns_record_counted_sites_and_hold_both_directions`
  - `ledger_pattern_declarations_are_closed_referenced_and_not_executable_by_presence`
  - `ledger_core_nodes_refuse_unknown_missing_and_orphan_assertions_with_valid_neighbours` (ledger vocabulary pin)
  - `ratcheted_sites_grow_on_new_keys_and_rising_counts_and_lower_on_falls_and_stale_rows`
  - `invalid_groups_and_zero_copy_rows_are_refused`
  - `counted_nodes_refuse_redundant_counts_and_unowned_assertions`
  - `both_native_spellings_round_trip_and_input_order_does_not_change_bytes`
  - `canonical_reports_preserve_golden_output_documentation_and_roundtrip` (census vocabulary pin)
- `cargo test --locked -p cargo-census --test delivery -- debt_command_adopts_refuses_growth`
  exited 0 (`recheck-delivery.log`). It ran 1 test.
- Hook log `/home/paudley/Active/rust-agentic-foundations/.stage/209-proof/fix-commit3.log`:
  repo_gates 29, docs_contract 81, self_census 4 (including
  `the_repository_debt_record_round_trips_byte_for_byte`) and self_workplan 1 all passed,
  and commit `53f5425f` was created.
- `prose-check` on the 4 documents changed by the fix and on the fix commit message
  exited 0. A grep of the added lines in `21e8c277..53f5425f` found no tracker numbers or
  deferral wording.
- `git diff 21e8c277..53f5425f` shows 0 capability `state:` changes and no change to
  `tests/docs_contract.rs`. `work/changes/` holds only two records that belong to other
  changes.

## Recheck of the prior findings

### 1. REQUIRED, purrdf framing: resolved

`crates/raf-census/src/report/site.rs` now builds the key as follows:

- `const DOMAIN: Domain = Domain::new(b"raf-census/debt-site/v1")`;
- `hasher.update(DOMAIN.as_bytes())`;
- `frame_le_into(&mut hasher, symbol.as_bytes())`;
- then the presence byte: `0`, or `1` followed by `frame_le_into(&mut hasher, literal.as_bytes())`.

`length_prefixed` and its u32 refusal are gone, and `SiteKey::digest` is now infallible
and `#[must_use]`. Every caller was updated: `gate/ratchet.rs` and the three test files.

Request (a), whether the key is correct:

- The domain is a fixed raw prefix. This is one of the two forms purrdf's `frame_le_into`
  documentation allows ("as raw leading bytes").
- The symbol and literal are injective u64 little-endian frames, and the presence byte
  separates `None` from `Some("")`. The existing `assert_ne!` covers that case.
- So the preimage is unambiguous.

Request (a), whether `docs/architecture/normal-form.md` matches the code exactly: it does.
It describes the same steps in the same order: the domain bytes as a `purrdf_hash::Domain`,
then the symbol as a `frame_le_into` field (u64 little-endian length and bytes), then one
presence byte, then the same framing of the literal when the byte is 1. The doc comment in
`site.rs` agrees. No document still describes the u32 framing for site keys. The remaining
u32 references are the frozen fingerprint encoding.

### 2. REQUIRED, `DebtWrite.sites` meaning: resolved

`write_debt_with` now reports `report.debt.sites().len()`, which matches the
`groups`/`jobs` row convention. The field doc says "distinct ratcheted site rows". The
Display text "N ratcheted sites", `census-debt-ratchet.md` and `census-rule-api.md` now all
say the same thing.

### 3. REQUIRED, zero-hit guard test: resolved

`burned_down_ratchet_stays_a_guard` extends the existing gate case. `ratchet_tree` now takes
the home body, so the fixture can remove every hit, the home's included. The test shows:

- check reports exactly 3 findings, all "stale site";
- the lowered record contains `ratchetedRule` and no `SiteDebt`;
- re-checking with that record is clean, with no "matches no source item" finding;
- adding `RATCHET_ONE` back gives exactly 2 growth findings.

From reading the code, without the `stale_matchers` skip this test fails. `rules::collect`
seeds every referenced id with an empty map, so the stale finding would make the first
count 4 and the guarded check unclean. The test now protects the behaviour it is bound to.

Request (c), whether "debt --write lowers the record to the rule name with no site rows" is
exercised:

- The test takes the lowered record from `check_tree(...).debt()`, not from the writer.
- The writer, `write_debt_with` (via `write_debt`), evaluates through
  `evaluate_reference_tree`. That is the same evaluation `check_tree` uses (it differs only
  in the host-root argument for reference observations), and it does not run `check_debt`.
- The writer refuses only on evaluation findings. The burned-down evaluation has none: all
  3 findings in the test come from `check_debt`, and the guarded check is clean.
- The writer refuses only on growth comparisons. A stale `Site { present: 0 }` is not
  growth, as `ratcheted_sites_grow_on_new_keys...` asserts.
- It then publishes `report.debt.to_turtle()`, the same record whose bytes the test checks.
- The CLI delivery test runs the real disk write path for a lowering (3 sites to 2,
  then a clean check).

So the claim is covered by composition: the same evaluation, the same refusal conditions,
the same record and the same publication path. A disk write of a zero-site record is not
run end to end. That would be cheap to add to the CLI case (OPTIONAL A below), but nothing
in the claim depends on a different path.

### Optional findings 4 to 7: all addressed

- **4.** `Adopt` now has its own direction text ("to adopt the newly ratcheted rule into the
  record").
- **5.** `count_sites` uses `expect("INVARIANT: ...")` in place of `unwrap_or_default()`.
- **6.** `with_ratchet` parses each `ratchetedRule` as a `RuleId`. A new refusal case
  (`"bad id"`) sits beside a valid neighbour (`"other"`) in
  `counted_nodes_refuse_redundant_counts_and_unowned_assertions`.
- **7.** The blank line before `## Version-bump rule` is restored. The `debt-ratchet.md`
  line is reflowed. The Examples paragraph now carries its own `raf:tested-by`, and the
  verification matrix row lists Examples. The contract states the equal-key swap case
  outright.

## Renewed assessments in the fix commit (request b)

The fix commit renews 18 assessments. I read each one's section at `53f5425f` against the
fix.

- **census-debt-ratchet: contract, examples, constraints, purpose, usage, explanation,
  verification.** Only the contract and examples sections changed. Contract now covers
  row-count `DebtWrite` and the equal-key swap case. Examples now covers the burned-down
  guard, with its binding. Both match the code. The other five were renewed for changed
  symbol digests (`DebtWrite`, `SiteKey`, `write_debt_with`, `gate::ratchet`,
  `report::debt`). Their text makes no claim about the framing, the row count or the
  guard, and it stays accurate.
- **census-rule-api contract.** The section changed to "the number of distinct recorded
  ratcheted site rows", which matches `write_debt_with`.
- **census-patterns, all seven sections.** These were renewed only for the
  `raf_census::gate::ratchet` digest (the `expect` and the infallible `digest` call). The
  Ratcheted patterns paragraph ("not findings ... counted site ... stays a recorded guard
  rather than going stale ... A literalRegex hit is keyed on its own literal") is still
  accurate and is now actually tested. The other sections do not touch the ratchet.
- **census-normal-form: purpose, examples, verification.** These were renewed only for the
  `raf_census::report` module digest (the `site.rs` and `debt.rs` changes). The capability
  sections describe the normal form, fingerprints and report-form regeneration. They say
  nothing about site keys, which only the architecture document `normal-form.md`
  describes, and they stay accurate.

None is inaccurate or incomplete for the delivered behaviour.

## Findings

No REQUIRED findings remain.

- **OPTIONAL A.** Extend `debt_command_adopts_refuses_growth_and_records_shrink_through_the_real_gate`
  by one step: remove every hit (`ratcheted_source(root, &[])`), run `debt --write`, and
  check that the published file holds `ratchetedRule` with no `SiteDebt`. This would show
  the zero-site write on disk end to end. The current coverage holds by composition, as
  set out under request (c).
- **OPTIONAL B.** Some lines added by the fix in `census-debt-ratchet.md` (the equal-key
  sentence) and `census-rule-api.md` are not reflowed to the surrounding width. This is
  cosmetic only.

VERDICT: PASS
