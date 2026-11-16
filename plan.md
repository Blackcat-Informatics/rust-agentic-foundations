# Plan: counted per-file ratchet, step 2 of 2 (rules and built-in detectors)

Base 106e42d5. Step 1 (patterns) is on main; this step reuses its site key, site records, compare and write paths.

## Design (line numbers at 106e42d5)

1. Ledger term: `ratchetedRule` on a LedgerJob, a repeatable nonempty string naming a rule id. It matches the DebtRecord's existing `ratchetedRule` property (the debt record already carries ratcheted rule ids as plain strings, so rules need no new report property). Patterns keep the boolean `ratcheted` on the Pattern.
   - `report/vocab.rs` LEDGER_FIELDS LedgerJob (~:176) gains `ratchetedRule`.
   - `report/ledger.rs`: `LedgerJob.ratcheted_rules: Vec<String>` (:100 area), read in `read_job` (:402), duplicates refused (`E::Repeated`), new `Ledger::ratcheted_rules()` (beside `patterns()` :173).
   - `report/ledger/declarations.rs`: `ratcheted_rules(jobs)` called from `Ledger` load before patterns. Refusals, each with an actionable reason: the id names a pattern (ratchet it on the pattern); the job does not forbid the rule (not enabled: this also covers unknown ids, since an enabled unknown id is already refused by the gate with RegistrationRequired); the job is not enforced; the detector is job-scoped; another job forbids the same rule outright (one rule, one meaning across jobs).
   - `detectors/mod.rs`: `job_scoped(id)` (home-literal, vocabulary-literal, sole-name, foreign-host-import), now also used by `gate/rules.rs` `requested` for the existing "requires a ledger job and home" refusal. These four judge one job's home and context and have no per-file site, so they cannot be ratcheted.
2. Hit positions (`gate/rules.rs`): `Offsets` entries become `Placed { byte, line, message }` (was a (byte, line) pair at :17). Syntax built-ins already record the exact byte (`record_hit`, `source_hit`). Fact detectors (hex-table, json-pointer-escape; `fact_hits` :233) and registered rules (`collect`, after `clone_from` :160) report a line only, so `record_line` derives the byte: the start of the hit's line, no earlier than the owning unit's start. It runs only for ratcheted ids. The public `Rule` trait and `RuleHit` are unchanged; the rule API states that a ratcheted rule's site is the owning item plus the first string literal at or after the start of the hit's line within that item.
3. Gate (`gate/ratchet.rs`, `gate/evaluate.rs`): `Sink::new` adds `ledger.ratcheted_rules()` to the ratcheted set (so `report_copies`, the COPY suppression and `collect_debt` already route rules like patterns). Diverted hits are keyed (rule, unit, ordinal) with byte, line and message, so two hits on one line count twice and a rule reached by two jobs counts once; locations use the hit's own message (patterns: the summary, as before). A rule that is both `required` on the Gate and ratcheted by a job is refused at evaluation (a required rule is forbidden everywhere).
4. Check and write: unchanged `DebtRecord::compare`, `check_debt`, `write_debt_with`. Rules are in the record's `ratchetedRule` set, so adopt, release, new site, rising, falling and stale behave as for patterns.
5. Reports: `DebtWrite.sites` exists. `RegisteredDebtWrite` (`registered/debt.rs` :18) gains an optional `siteCount` (positive integer, written only when the count is positive) and `sites()`; older readers refuse the unknown property. Protocol version stays.

## Report-form decision

No REPORT_FORM or NORMAL_FORM bump. The debt record needs no new property: `ratchetedRule` and `SiteDebt` are already optional and omitted at default, and rule ids are strings. A record with no ratcheted rule is byte-identical. The ledger gains the optional `ratchetedRule` on LedgerJob (additive, omitted at default; an older reader refuses the unknown property by name). The receipt gains the optional `siteCount`.

## Tests (extend existing cases)

- `census_gate.rs`: extend `ratcheted_patterns_record_counted_sites_and_hold_both_directions` neighbourhood with a built-in detector case and a registered-rule case (adopt exact sites, clean, line insertion byte-identical, extra hit fails check and write, removed hit fails until write, deleted file stale, forbidden neighbour hard fails and cannot be adopted); extend the ledger declaration refusal case with `ratchetedRule` refusals beside a valid neighbour; extend the ledger vocabulary inventory to emit `ratchetedRule`.
- `debt.rs`: no new case needed (record semantics are rule-agnostic), unless the receipt needs it.
- `cargo-census/tests/delivery.rs`: extend the debt command case with a ratcheted built-in detector on disk.
- `registered/tests.rs`: extend the receipt round-trip, refusals and vocabulary inventory with `siteCount`.

## Documents

Change record `work/changes/census-rule-ratchet.md`; capability documents census-debt-ratchet, census-ledger, census-rule-api, census-detectors, rdf-report-profile (LedgerJob row, RegisteredDebtWrite row); proposed blocks, then delivered text; renew affected assessments after reading; complete the change.

## What step 2 cannot honour

- Job-scoped detectors (home-literal, vocabulary-literal, sole-name, foreign-host-import): their hits belong to one job's home and context and carry no per-file key independent of the job; refused at ledger load.
- A registered rule's site is line-granular (its hit states a line, not a column); two hits on one line share a start and count separately.
- Key churn on a rename of the enclosing item or the keyed literal, as in step 1.
