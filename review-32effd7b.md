# Delivery review: 32effd7b (counted site ratchet for patterns, step 1 of 2)

## Scope

- Head reviewed: `32effd7b15a961cd13220ad31b74ce54476dda8e` on
  `paudley/209-census-debt-a-counted-per-file-ratchet`, range `21e8c277..32effd7b`
  (64 files, +1546/-244).
- Requirement: `.stage/census-debt-a-counted-per-file-ratchet/issue.md` (verbatim issue,
  no comments). Plan and `## User rulings`: `.stage/census-debt-a-counted-per-file-ratchet/plan.md`.
  The only ruling is the coordination ordering, plus "byte-identical golden before the
  merge". Step 2 (job-level `ratchetedRule` for built-in and registered rules,
  `RegisteredDebtWrite`) is a separate landing under the plan. It is not judged here.
- Read: every source, test, document and assessment hunk in the range. Also the callers
  `report_copies`, `stale_matchers`, `collect_debt`, `check_debt`, `write_debt_with`,
  `Workspace::item_at`, `gate/context.rs` and `registered/debt.rs`, plus the purrdf-hash
  4.0.2 `domain.rs` and `frame.rs` that RAF resolves (located with
  `cargo metadata --offline --locked`).

## Evidence

My own runs, each under `flock /tmp/raf-build.lock` from the worktree. Logs are in
`/home/paudley/Active/rust-agentic-foundations/.stage/209-reviews/`:

- `cargo test --locked -p raf-census --test census_gate --test debt --test reports -- ratchet ledger_pattern_declarations ... both_native invalid_groups`
  exited 0 (`focused-raf-census.log`). It ran 6 tests:
  `ratcheted_patterns_record_counted_sites_and_hold_both_directions`,
  `ledger_pattern_declarations_are_closed_referenced_and_not_executable_by_presence`,
  `ratcheted_sites_grow_on_new_keys_and_rising_counts_and_lower_on_falls_and_stale_rows`,
  `invalid_groups_and_zero_copy_rows_are_refused`,
  `counted_nodes_refuse_redundant_counts_and_unowned_assertions` and
  `both_native_spellings_round_trip_and_input_order_does_not_change_bytes`. The reports
  filter selected zero tests, so I re-ran that check separately (next item).
- `cargo test --locked -p raf-census --test census_gate --test reports -- ledger_core_nodes_refuse_unknown canonical_reports_preserve_golden`
  exited 0 (`focused-vocab.log`). It ran 2 tests: the two vocabulary pins, which compare
  the emitted terms with the `rdf-report-profile.md` tables.
- `cargo test --locked -p cargo-census --test delivery -- debt_command_adopts_refuses_growth`
  exited 0 (`focused-delivery.log`). It ran 1 test, the real CLI path: adopt, hold, fourth
  hit refused by both `check` and `debt --write` with the file unchanged, fall, lower,
  settle.
- Hook run `/home/paudley/Active/rust-agentic-foundations/.stage/209-proof/commit3.log`:
  repo_gates 29, docs_contract 81, self_census 4 (this includes
  `the_repository_debt_record_round_trips_byte_for_byte`) and self_workplan 1 all passed,
  and the commit was created. The author's whole-crate logs (`census-all.log`,
  `delivery2.log`, `clippy3.log`) record no command or exit status. I treat them as
  claims only.
- `prose-check` on the 7 changed documents and on the commit message exited 0. A grep of
  the added lines found no tracker numbers, process wording or deferral wording.
- `git diff` shows no `state:` line change in any capability document, no change to
  `tests/docs_contract.rs`, and no change record left in `work/changes/`. The two files
  there belong to other changes.

Acceptance mapping (plan step 1 against the issue):

| Acceptance | Production path | Evidence |
|---|---|---|
| Three hits in two files record those exact sites | `report_copies` diverts to `ratchet::Sink`, `collect_debt` calls `with_ratchet` | gate test asserts 3 sites in `src/a.rs`, `src/a.rs`, `src/b.rs`, copies 1; CLI prints "3 ratcheted sites" |
| A fourth hit fails check and write (new key and repeated key) | `DebtChange::Site` growth, `check_debt`, `write_debt_with` | gate test checks both keys; CLI checks both commands and the unchanged file |
| Inserting lines leaves the record byte-identical | key preimage has no line or offset | gate test compares the `to_turtle()` bytes |
| A removed hit fails check until `debt --write` lowers it | `Site` shrink, then write | gate test and CLI test |
| A deleted file is stale | `Site { present: 0 }` gives "stale site" | gate test removes the module and its `mod` declaration |
| Forbidden patterns keep hard failure | non-ratcheted ids keep the existing finding path | gate neighbour: 3 rule findings, no sites; ledger refusals with valid neighbours |
| A record without ratcheted rules is byte-identical | `ratchetedRule` and `SiteDebt` are written only when non-empty | self_census golden on the real `census-debt.ttl` |

What I confirmed:

- **Shared literal scan.** `string_literals` is factored out of `gate/patterns/mod.rs`, and
  both literalRegex and `gate/ratchet.rs` call it. It charges `MatchLimits` and refuses
  when a bound is exceeded. Files are lexed only when they hold a ratcheted hit.
- **Site identity.** The key has no line number or byte offset. Multiplicity is counted per
  key, and a different-key swap is one missing key plus one new key.
- **RDF.** `SiteDebt` is a closed shape (four 1..1 properties). The new object type
  "site key" accepts only 64 lowercase hex digits. The reader recomputes the node name, and
  a site row needs its rule in `ratchetedRule`. Turtle and N-Triples round-trip, and input
  order does not change the bytes. No REPORT_FORM or normal_form bump is needed under the
  additive-property rule in `normal-form.md`.
- **Ledger flag.** The true-only reader is shared with `shippingOnly` (`ledger/flag.rs`)
  rather than copied.
- **Renewed assessments.** I read all 35 of them against their sections. They cover
  census-cli (5), census-debt-ratchet (7), census-ledger (6), census-normal-form (3),
  census-patterns (7), census-rule-api (6) and rdf-report-profile contract (1). Apart from
  findings 2 and 3, each section's text is accurate for the delivered behaviour. The
  census-cli, normal-form and most ledger, patterns and rule-api renewals were triggered
  only by changed symbol digests, and their sections make no claim that the change
  falsifies.

## Findings

### 1. REQUIRED: the site-key preimage re-implements purrdf-hash framing and domain

`crates/raf-census/src/report/site.rs` passes raw domain bytes `b"raf-census/debt-site/v1"`
to the BLAKE3 hasher. It also frames each field with a local `length_prefixed` (u32
little-endian length plus bytes). The purrdf-hash crate that RAF resolves (=4, 4.0.2)
already supplies both:

- `purrdf_hash::Domain`, "the one type every domain-separation string ... is spelt through";
- `purrdf_hash::frame::frame_le_into`, "the one way a variable-length field enters a
  preimage". It frames with a u64 little-endian length and works with
  `blake3::Hasher` (`Streaming<16384>` implements `Digest`).

AGENTS.md section 6 says: "Use the purrdf.* libraries for every reusable function they
supply ... wire the real callers through the native API, and remove the duplicate." The
plan's precedent, `normalize/fingerprint.rs`, is a frozen published encoding. This key is
not published yet. Once it merges, every consumer debt record stores these digests, and any
later change to the preimage moves every recorded key. Now is the cheap moment to fix it.

Required correction:

- Declare `const DOMAIN: Domain = Domain::new(b"raf-census/debt-site/v1")`.
- Absorb `DOMAIN.as_bytes()` into the hasher.
- Absorb the symbol and the literal with `frame_le_into`, keeping the presence byte.
- Delete `length_prefixed` and its u32 refusal (`SiteKey::digest` can become infallible).
- Update the preimage text in the `site.rs` doc comment and in
  `docs/architecture/normal-form.md` (it currently says "a little-endian u32 length").

### 2. REQUIRED: `DebtWrite.sites` counts hits, but the contract says sites

`write_debt_with` sets `sites: report.debt.sites().values().sum()`. That is the sum of the
`copies` values, which is the hit count. The field doc says "number of ratcheted pattern
hits", but Display prints "N ratcheted sites". `census-debt-ratchet.md` (Counted site
ratchet) and `census-rule-api.md` (`.write_debt()`) both say `DebtWrite` reports "the
number of recorded ratcheted sites".

When one key has copies greater than 1, the summary line and both renewed contract sections
overstate the number of sites. The sibling fields `groups` and `jobs` count rows.

Required correction: either report `sites().len()`, matching the `groups`/`jobs`
convention, or rename the reported quantity to hits in the code, the Display text and both
documents.

### 3. REQUIRED: the zero-hit "stays a guard" behaviour is claimed and bound to a test that cannot exercise it

`stale_matchers` now skips ratcheted ids, so that a burned-down pattern does not become
"matches no source item". `census-patterns.md` claims this ("A ratcheted pattern with no
hit stays a recorded guard rather than going stale") and binds the claim to
`ratcheted_patterns_record_counted_sites_and_hold_both_directions`. `census-debt-ratchet.md`
repeats it.

That fixture's `src/lib.rs` home always contains `std::mem::drop(0)`. So the pattern's hit
map is never empty, and the stale check would not fire even without the skip. The
`debt.rs` guard case covers only `DebtRecord::compare`, not the gate.

Burning down to zero is the end state of the issue's per-file scenario. Required
correction: extend the existing gate case with a step where no hit remains anywhere,
including the home. It should show:

- check fails with stale sites;
- `debt --write` lowers the record to `ratchetedRule` with no `SiteDebt` row;
- the next check is clean, with no "matches no source item" finding;
- a new hit is then growth.

### 4. OPTIONAL: the Adopt finding says "lower the record"

`check_debt` falls through to the default direction for `DebtChange::Adopt`, so the finding
reads "...to lower the record". The action is right, but adopting records new sites.
A dedicated message ("to adopt the ratcheted rule's present sites") would be accurate.

### 5. OPTIONAL: `count_sites` uses `summaries.get(rule).copied().unwrap_or_default()`

The rule always comes from the ledger's ratcheted patterns, so the fallback cannot trigger.
An explicit `INVARIANT:` lookup would follow the repository's no-silent-fallback style.

### 6. OPTIONAL: `with_ratchet` checks only that `ratchetedRule` values are non-empty

Site rows are parsed as `RuleId`, but rule names are not. A hand-edited
`ratchetedRule "bad id"` loads and then shows up as a `Release`. Parsing the rule names as
`RuleId` would close the profile fully.

### 7. OPTIONAL: documentation layout

- `docs/architecture/normal-form.md` has no blank line between the new site-key paragraph
  and `## Version-bump rule`.
- `docs/architecture/debt-ratchet.md` has one over-long reflowed line.
- The new Examples paragraph in `census-debt-ratchet.md` has no `raf:tested-by` of its own.
- The Counted site ratchet text could say outright that two hits sharing a key can swap
  without a change. It is implied by "a swap of two hits with different keys", but it is
  the one case the key does not separate.

Not findings:

- `RegisteredDebtWrite` does not carry `sites`. The plan assigns `RegisteredDebtWrite` to
  step 2.
- Re-ratcheting resets the allowance. This is visible in the ledger and the debt diff, and
  is recorded as a plan risk.

VERDICT: BLOCKED
