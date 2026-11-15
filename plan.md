# Plan: counted per-file ratchet for rule and pattern hits (issue 209), step 1 of 2

Base: origin/main 21e8c277. Worktree: .worktrees/209-census-debt-a-counted-per-file-ratchet. Build lock: /tmp/raf-build.lock.
Verbatim issue: .stage/census-debt-a-counted-per-file-ratchet/issue.md in this worktree.

## Behavior (step 1: patterns)

A ledger Pattern may carry `ratcheted true`. Its hits are not findings. They are recorded in the debt record as counted site rows keyed by (file, rule id, site key). The two-sided ratchet applies: a new site or a rising count fails check and write; a falling count fails check until `debt --write` lowers the record; a recorded site whose site is gone (including a deleted file) fails as stale; the record carries no unused allowance. Forbidden (non-ratcheted) rules and patterns keep hard failure and cannot be adopted as debt.

## Dependencies and exclusions

- Step 2 (separate delivery, same issue, lands right after step 1): ratcheting built-in detectors and registered rules through a job-level `ratchetedRule`, offsets for registered-rule hits, `RegisteredDebtWrite`. Step 1 leaves this out and the PR refers to the issue without closing it.
- No normal_form bump and no REPORT_FORM bump (stays 5). Additive and optional: a record with no ratcheted rule stays byte-identical.
- Out of scope: any softer rule for a renamed enclosing symbol (see Risk).

## Design (from the read-only design assessment; line numbers at 21e8c277)

1. Vocabulary. Add `ratcheted` to the Pattern field list in `crates/raf-census/src/report/vocab.rs` (LEDGER_FIELDS, about lines 213-230). Boolean stated only as true, the convention `shippingOnly` uses in `report/ledger/scope.rs:89-101`: extract that true-only read into one shared helper under `report/ledger/` and call it from both. `LedgerPattern` (`report/ledger/declarations.rs:32-41`, built at about :115) gains `pub ratcheted: bool`. Refusals at ledger load, each beside a valid neighbour: `ratcheted false` or a non-boolean; `ratcheted` stated twice; a ratcheted pattern referenced by a job with `enforced false` (check `job.patterns` and rule ids in `declarations::patterns`). Unknown properties already refuse in `ledger.rs` fields().
2. Site identity. New `report/site.rs` with `SiteKey([u8; 32])` (`digest(symbol, Option<&str>)`, `to_hex`, `from_hex` via the existing canonical hex decoder used in `embedded/schema.rs:419`) and `DebtSite { file, rule, key }` (Ord). Re-export from `report/mod.rs` and `lib.rs`. Digest with `purrdf_hash` (Blake3 Hasher, Domain, hex::Lower) like `normalize/fingerprint.rs:11-26` and `embedded/debt/rdf.rs:27-37`. Preimage: domain bytes `raf-census/debt-site/v1`, then u32le length plus symbol bytes, one presence byte (0 or 1), and when present u32le length plus the literal bytes. File and rule are not hashed. Line numbers are never hashed. The symbol is the owning unit or item symbol (first element of UnitKey, `gate/rules.rs:13`, taken as in `source_hit` `rules.rs:375-379`). The literal is the decoded value of the first ordinary or raw string literal whose start byte is at or after the hit byte and before the end of the enclosing item (`workspace.item_at(file, byte)` and its span end); byte and C strings and desugared doc strings are excluded by the same written-literal test as `gate/patterns/mod.rs:121`. A literalRegex hit keys on its own literal.
3. Literal scan. Factor the lex plus work-list walk at `gate/patterns/mod.rs:94-146` into one `pub(super) fn string_literals(label, text, limits) -> Vec<(start_byte, String)>`; literalRegex and the new `gate/ratchet.rs` both call it (no copy). Charge it to MatchLimits through the existing charge and push; lex only files that have a ratcheted hit; exceeding a bound refuses, never truncates.
4. Multiplicity. Hits sharing (file, rule, key) are counted; the row stores `copies >= 1`. A swap of two hits with different keys is one missing key plus one new key, so a removed hit cannot hide a new one.
5. Hit offsets. `RuleHit` is public: do not change it. Add an internal `offsets: rule -> UnitKey -> Vec<(byte, line)>` beside `subjects` in `rules::Collected`, filled at `record_hit` (`rules.rs:320-330`) and `source_hit` (`rules.rs:389-398`).
6. Gate. In `report_copies` (`gate/evaluate.rs:395-422`) divert hits of a ratcheted id from findings into a BTreeSet of (rule, UnitKey, byte) (dedups two jobs sharing a pattern); keep the generic COPY finding suppressed. `stale_matchers` (`evaluate.rs:592-599`) skips ratcheted ids so a burned-down pattern with zero hits stays a guard. Compute digests once in `collect_debt` (`evaluate.rs:145`). `GateReport` keeps a private map DebtSite -> Vec of `file:line: symbol: message` so growth findings name the location.
7. Debt record. `DebtRecord` (`report/debt.rs:127-132`) gains `ratcheted: BTreeSet<String>` and `sites: BTreeMap<DebtSite, usize>`. `DebtRecord::new` keeps its signature; add `with_ratchet(rules, sites)` shaped like `scoped` (`debt.rs:172`), refusing a site whose rule is not in `rules`, an empty file or rule, a zero count, an invalid RelPath or RuleId. RDF profile: optional `ratchetedRule` on the DebtRecord root (0..n string, written only when non-empty); new `SiteDebt` node with `file`, `ruleId`, `siteKey` (new object type "site key": 64 lowercase hex), `copies` (positive integer). Node name `#site/{mint::part(rule)}/{hex}@{percent::encode(file, PATH)}` via a new `mint::site`; the reader recomputes the name and errors on mismatch like `debt.rs:403`/`:410`. Add the SHAPES rows after `vocab.rs:145`, a "site key" arm in the object match at `shape.rs:206-245`, a `"SiteDebt"` arm in `DebtRecord::from_graph` (`debt.rs:371-421`) and a render loop in `render` (`debt.rs:290-357`). Cross-checks on read: the row's rule is in `ratchetedRule`; no duplicate rows. `ratchetedRule` is required so a newly ratcheted pattern can be adopted into an existing record and an emptied rule stays a guard.
8. Compare. Extend `DebtRecord::compare` (`debt.rs:207-256`), `DebtChange` and `is_growth` (`debt.rs:428-471`): rule present but not recorded: `Adopt{rule}` (not growth; check fails until write; its sites not compared). Rule recorded but not present: `Release{rule}` (not growth; fails until write; write drops rows). Rule in both, per (file, rule, key): present > recorded: `Site` growth; present < recorded: `Site` shrink; present == 0: stale. Update the matcher at `gate/mod.rs:631-641` (check_debt) with "stale site ... run `debt --write` to lower the record". `write_debt_with` (`gate/mod.rs:404-416`) already refuses growth. `DebtWrite` (`gate/mod.rs:47-68`, non_exhaustive) gains `sites: usize`, and Display appends `, N ratcheted sites` only when N > 0. An absent record is empty. `RegisteredDebtWrite` unchanged.

## Acceptance (issue 209, step 1 scope)

- Adopting a ratcheted pattern with three hits in two files records those exact sites.
- A fourth hit fails both check and write (a new key, and a repeated key).
- Inserting lines above a recorded site leaves the record byte-identical.
- Removing a hit fails check until `debt --write` lowers the record, after which check passes.
- A recorded site whose file is deleted is stale.
- Forbidden rules and patterns keep hard failure and still cannot be adopted as debt (neighbour fixture).
- A record with no ratcheted rule stays byte-identical: golden `DebtRecord::from_turtle(file).to_turtle() == file` for the repository's own `census-debt.ttl` in `tests/self_census.rs`.

## Tests (extend existing cases; every refusal beside a valid neighbour)

1. `crates/raf-census/tests/census_gate.rs`: one new test `ratcheted_patterns_record_counted_sites_and_hold_both_directions` based on `scoped_tree` (about :4650-4698) covering adoption, clean with the record present, line insertion byte-identical, fourth hit (new key and repeated key) growth, removed hit lower-the-record, deleted file stale, and the non-ratcheted neighbour producing findings with no sites.
2. Extend `ledger_pattern_declarations_are_closed_referenced_and_not_executable_by_presence` (about :451) with `ratcheted false`, repeated `ratcheted`, and an unenforced job, plus the valid `true` neighbour.
3. Extend `debt_command_adopts_refuses_growth_and_records_shrink_through_the_real_gate` (`crates/cargo-census/tests/delivery.rs` about :437-473) with a ratcheted fixture on disk.
4. `crates/raf-census/tests/debt.rs`: extend `counted_nodes_refuse_redundant_counts_and_unowned_assertions` (:121), the round-trip case (:256), and the growth (:171) and shrink (:226) cases with sites, adopt, release, stale.
5. `tests/self_census.rs`: the byte round-trip golden above.
6. Vocabulary pins: `census_vocabulary_reports` (`reports.rs` about :76-128) must build the debt fixture with `.with_ratchet(...)`, and the Ledger inventory fixture (`census_gate.rs` about :404) must emit `ratcheted`, because `tests/support/vocabulary.rs` compares emitted terms to the `rdf-report-profile.md` tables exactly.
No clock, no mutation testing, no discretionary extra tests.

## Documents (AGENTS.md section 7: change record, bindings, assessments)

`census-ledger.md` (the "cannot become recorded debt" sentence near :239 becomes: forbidden rule and pattern findings cannot become recorded debt; a ratcheted pattern's hits are recorded as counted sites; add a `ratcheted` sentence in the patterns paragraph near :263-266); `rdf-report-profile.md` (Pattern row near :211, SiteDebt and ratchetedRule near :196-202); `normal-form.md` (table near :440-461 and the additive-property paragraph near :515-525, stating that a new node class emitted only in a new mode is additive); `census-patterns.md`; `census-debt-ratchet.md` (new "Counted site ratchet" contract section, code and test bindings for `raf_census::DebtSite`, `raf_census::SiteKey` and the ratchet module); `census-rule-api.md` (:148-152 DebtWrite). Open `work/changes/<slug>.md`, renew affected assessments with `cargo workplan assess` only after reading each section against the change (restore the two licence comment lines), run `cargo workplan change complete`. `tests/docs_contract.rs` INVENTORY changes only if a state changes (none expected). No tracker numbers or process wording in any committed file.

## Risk

- Key churn on rename: renaming the enclosing function or editing the keyed literal makes one new key plus one missing key; the new key is growth, so write refuses. Decision: follow the issue literally; no softer swap rule. Flagged to coordination as a possible follow-up decision.
- `DebtChange` gains variants (greenfield; the only outside matcher is `gate/mod.rs:634`).
- Un-ratcheting and re-ratcheting a pattern resets its allowance (visible in the ledger and debt diff).
- Existing consumer ledgers never state `ratcheted`, so what a consumer census run accepts is unchanged; older binaries refuse a ledger that states it by property name.

## User rulings

- Coordination (standing direction as Patrick's): order 201, then 208 and 209, then the rest of 206 to 216; report each landing; tell Katamari before a merge that changes what its census run accepts; byte-identical golden before the 209 merge.
