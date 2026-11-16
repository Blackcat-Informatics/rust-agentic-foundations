<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Delivery review: 08ea247d (counted per-file ratchet, rules and built-in detectors)

## Scope

- Worktree `/home/paudley/Active/rust-agentic-foundations/.worktrees/209-census-debt-a-counted-per-file-ratchet`, range `106e42d5..08ea247d` (one commit, 61 files).
- Requirement: the verbatim issue at `.worktrees/209-census-debt-a-counted-per-file-ratchet/.stage/census-debt-a-counted-per-file-ratchet/issue.md`. The caller's path under the main checkout's `.stage/` does not exist; the issue was read from the worktree's `.stage/`. Step 1 (patterns) is on main. This step must deliver a job-level ratchet for built-in detectors and registered rules with the same counted-site semantics, offsets for registered-rule hits, site counts in `DebtWrite` and `RegisteredDebtWrite`, and an additive report form.
- Rulings: `step1-plan.md` `## User rulings` (order of landing, byte-identical golden before merge). `plan.md` for step 2 has no rulings section. No ruling cuts scope.
- Binding rules: AGENTS.md sections 5 to 8; rust-work skill.

## Evidence

Commands I ran (logs under `/home/paudley/Active/rust-agentic-foundations/.stage/209-reviews/`):

- `flock /tmp/raf-build.lock cargo test --locked -p raf-census --test census_gate -- ratchet a_required_rule ledger_pattern_declarations ledger_vocabulary an_unknown_rule_is_hard`, EXIT=0, 5 passed: both ratchet cases, the required-rule refusal, the ledger declaration case (which runs the new `ratchetedRule` refusals) and the registered-hit adoption refusal (`r08ea-census-gate.log`).
- `flock /tmp/raf-build.lock cargo test --locked -p raf-census --lib -- registered::`, EXIT=0, 4 passed, including the receipt round trip and the `siteCount` refusals (`r08ea-registered.log`).
- `flock /tmp/raf-build.lock cargo build --locked -q -p cargo-census`, EXIT=0. I then ran the built binary against two probe fixtures under `.stage/209-reviews/` (described in the findings).
- `prose-check` on the eight changed documents and on the commit message: EXIT=0 for both.
- A grep of the added lines for tracker numbers and process wording found nothing.

Implementer evidence I read: `s2-test-census_gate.log` (89 passed), `s2-test-debt.log` (13 passed), `s2-test-delivery.log` (20 passed), `s2-test-lib.log` (102 passed), `s2-clippy5.log` (`-p raf-census -p cargo-census --all-targets -D warnings`, EXIT=0), `s2-complete.log` (change `census-rule-ratchet` completed) and `s2-commit.log` (pre-commit hook with `self_census` and `self_workplan` passing, EXIT=0, commit 08ea247d). These runs used the working tree before the commit was made (HEAD 106e42d5 in each log header). My focused runs above are against 08ea247d itself.

Acceptance mapping (built-in `serde-json` and registered `flag-rule`, in `ratcheted_rules_record_counted_sites_and_hold_both_directions`; CLI in `debt_command_adopts_refuses_growth_and_records_shrink_through_the_real_gate` for `serde-json`):

- Three hits in two files are adopted as exactly three sites: covered.
- A fourth hit fails check and write, both as a new key and as a repeat on one line: covered (gate and CLI).
- Inserting lines leaves the record byte-identical: covered.
- A removed hit fails until the record is lowered, then passes: covered.
- A deleted file is stale: covered.
- A forbidden, unratcheted rule keeps its hard failure and cannot be adopted: covered for the direct case, but there is a hole (finding 1).
- `RegisteredDebtWrite` gains optional positive `siteCount`, refused when zero and on an error receipt: covered.
- Report form: no `REPORT_FORM` or normal-form bump. The ledger property and the receipt property are optional and omitted at their defaults. The debt record gains no property, and the self-Census round trip passed in the hook.

Callers and wiring: `Sink::new` chains `Ledger::ratcheted_rules()`, so `report_copies`, the COPY suppression and `collect_debt` route rules exactly as they route patterns. `rules::collect` records offsets for tracked registered rules (`record_line`) and tracked fact detectors (`fact_hits`). Syntax built-ins already recorded exact bytes. `requested` refuses a rule that is both Gate-required and ratcheted. `declarations::ratcheted_rules` runs at ledger load. `job_scoped` now has one definition, shared by the ledger refusal and the Gate's required-rule refusal. Changing the `Hits` key from byte to ordinal does not double-count pattern hits: path patterns already dedupe on (pattern, byte) in `patterns/paths/visit.rs`, and each pattern is collected once.

## Findings

### 1. REQUIRED: a forbidden pattern can be ratcheted through another job's `ratchetedRule`, which silently removes its hard failure

`declarations::ratcheted_rules` (`crates/raf-census/src/report/ledger/declarations.rs`) refuses a pattern id only when the same job lists it in `forbiddenPattern` (`job.patterns.contains(id)`). It runs before the patterns are read and cannot see the ledger's pattern ids. A job may also reference a pattern as `forbiddenFingerprint "rule:<pattern-id>"`, which `LedgerJob::referenced` and the Gate accept. The cross-job check looks only at rule fingerprints, so a job that names the pattern through `forbiddenPattern` is never compared.

Reproduction (`.stage/209-reviews/probe-pattern-via-rule`, control at `probe-pattern-via-rule-control`):

- Pattern `banned` (call `std::mem::drop`) does **not** state `ratcheted`.
- Job `number` is enforced and lists `forbiddenPattern "banned"`. Job `second` is enforced, lists `forbiddenFingerprint "rule:banned"`, and names `ratchetedRule "banned"`.
- Control without the `ratchetedRule`: `cargo-census check` reports both jobs' `rule banned` findings, EXIT=1.
- With it: the ledger loads. `check` drops both hard findings, including job `number`'s, and asks only to adopt (EXIT=1). `debt --write` records `1 ratcheted sites` (EXIT=0), and `check` then passes (EXIT=0).

This breaks the acceptance line "Forbidden (non-ratcheted) rules keep their current hard-failure behaviour and still cannot be adopted as debt". It also contradicts the delivered text in `docs/capabilities/census-ledger.md` ("The ledger refuses a ratchetedRule ... that names a pattern") and `census-patterns.md` ("that is the only way to ratchet a rule, and a pattern never uses it"). The same route skips step 1's rule that every job referencing a ratcheted pattern must be enforced: `Sink::is_ratcheted` is global, so an unenforced job's hits of that pattern would be diverted too.

Fix: refuse any `ratchetedRule` that names a declared ledger pattern id. Run the check after `declarations::patterns`, or pass the pattern ids in. Add the two-job case as a negative fixture beside its valid neighbour in `ratchet_rule_declarations_are_enabled_enforced_scoped_and_consistent`.

### 2. REQUIRED: the fact-detector ratchet path is claimed in the documents but no test runs it

`fact_hits` gained a tracked branch. It looks up the unit's source text, refuses when the text is absent, and calls `record_line`. `census-detectors.md` states that `hex-table` and `json-pointer-escape` sites use the start of the hit's line inside the owning item, and binds that section to `ratcheted_rules_record_counted_sites_and_hold_both_directions`. That test covers only `serde-json`, a syntax built-in with exact bytes, and `flag-rule`. None of the delivered tests ratchets a fact detector, so this production branch never runs in the suite.

I ran it through the CLI (`.stage/209-reviews/probe-fact`, two packages so that the home exemption does not apply; control `probe-fact-control` reports two hard findings). With `ratchetedRule "json-pointer-escape"`, `debt --write` records 2 sites. Inserting lines leaves the record byte-identical. A second escape on a recorded line is growth, and a removed escape is a stale site. The behaviour holds today, but the documented claim is unprotected.

Fix: add a fact-detector row to the existing ratchet loop, for example `("json-pointer-escape", "let _=\"~0\";")`. The fixture modules already sit outside the home package. Alternatively, drop the fact-detector sentence from the binding.

VERDICT: BLOCKED
