# Design review: raf-workplan strict mode (plan Rev 0)

Reviewed: `.stage/raf-workplan-strict-mode-force-coverage/plan.md` (contract) and `design.md` (D§n).

Code read: `crates/raf-workplan/src/{gate.rs,model.rs,load.rs,coverage.rs,finding.rs,obligation.rs,graph.rs,lifecycle/{complete.rs,untouched.rs,session.rs}}`, `crates/raf-workplan/tests/support/cases.rs`, `crates/raf-workplan/Cargo.toml`, `crates/raf-okf/src/{profile/schema.rs,profile/typed.rs,body/marker.rs}`, `crates/raf-okf/tests/{claims.rs,schema.rs}`, `crates/raf-source/src/{resolve.rs,parse.rs,item.rs,facts.rs,workspace.rs,lib.rs}`, `crates/cargo-workplan/src/{main.rs,coverage.rs}`, `tests/docs_contract.rs`, `docs/capabilities/census-debt-ratchet.md`.

## Judgments that hold

* **Setting in the coverage record.** It is the one repository-level policy record `check` already requires (coverage.rs:563-577), its schema is a closed table (schema.rs:152-159) so a misspelt value fails hard, and `max(record, floor)` has no way to lower. Sound.
* **`raf:tested-by` as a third marker family, not an extension of `raf:claim`.** `family_of` already routes any unknown `raf:` word to `UnknownMarker` (marker.rs:167-179, 288), so today's documents cannot contain the word and the parse fixtures of claims.rs:157-212 stay as they are. Sound.
* **Orphan symbols by widening the existing coverage kinds.** The gap, the fix and the exclusion mechanism are the same (coverage.rs:638-651); a second kind would be a duplicate critical path. Sound in intent; the seam it is placed on is wrong (B1, B2).
* **Debt as an OKF `Debt` record.** The row rules (`okf-row-unknown-key`, `okf-duplicate-entry`, `okf-bad-measure`, `okf-singleton-id`) and `record::create/replace` already exist; a JSON file would be a second parser. The semantics copy the census contract exactly (census-debt-ratchet.md:45-85: growth fails, shrinkage fails until the write, the write refuses growth except on adoption, the record always equals the present debt). `DocType` is exhaustive by design (schema.rs:165-169), so the one pin update is the honest cost. Sound.
* **Reverse coverage depends on T5.** `module_named` (resolve.rs:1360-1380) finds roots by the first path segment and then looks the full path up in `module_by_path`; a test or binary module is not there, so `resolve_from` from one is `UnknownModule`. T5 before T6 is the right order. `UnresolvedImpl` is already public with `span` (item.rs:244-265); `impls()` for the resolved ones is the missing half, and `Item.span`/`Item.body` (item.rs:300-302) give the per-item ranges. Sound.
* **Task order.** T1 moves the listed capabilities to `implementing` with their PROPOSED blocks and updates `INVENTORY` in the same commit; nothing judges the repository strict until T8 sets the key; each task adds its kinds and their `Case`s together, so `kinds.rs` both-directions stays green per commit. Sound.
* **Kinds count.** Twelve own kinds, five profile rules, two parse errors, two obligation kinds; each names a distinct fact with a distinct fix. None is a duplicate of a delivered kind.

## Findings

### BLOCKING

**B1. The strictness floor cannot reach the orphan-symbol widening as designed.**

Plan: "Orphan symbols: no new kind; strict widens the existing `coverage-item/test/reexport-unbound` from `scope` to every package a binding resolves into" (plan.md:46-47). D§3.3: "`coverage::check` gets `strictness` from `plan.strictness()`". D§2.2: "Strict findings are computed lazily in `check`, never in `relate`", with the floor applied by `Workplan::with_strictness(self, floor)` after load and by `Opened::workplan()`.

Code: `load::load` runs `plan.relate()` at load.rs:39, and `relate` runs `coverage::check` at model.rs:276. `Opened::workplan()` is `Workplan::load_with(&self.tree, &self.workspace, &self.links())` (gate.rs:423-425) with no floor parameter. So at the moment `coverage::check` runs, only the record's `strictness` is known; a floor applied afterwards by `with_strictness` or carried by `Opened` is invisible to it.

Consequence: `Gate::strict()` / `--strict` on a repository whose record says `default` raises the content rules and silently skips R4; the strict `Case`s of D§3.10, judged with `with_strictness(Strict)` after load, can never produce the widened gaps, so the R4 fixture and the both-directions test are unreachable as written. Two sources of truth for one setting.

REWRITE TO: keep `relate` strictness-free. `strict::check(plan) -> Vec<Owed>` computes the widened set itself: it collects the bound packages from `plan.graph().bindings()`, calls the existing `coverage::compute(plan.source(), plan.graph(), Some(record), &bound_not_in_scope)` (coverage.rs:552, 597), and turns each gap of a package outside `scope` into an `Owed` with the strict message. `coverage::check` and the default path do not change. With no coverage record loaded, `strict::check` judges no widening and no ceilings (`coverage-record-missing` already fails). Drop the claim in D§3.3 that `coverage::check` reads `plan.strictness()`.

**B2. Strict-only coverage gaps need a structural tag, not a message difference, to be debt.**

D§3.3: the widened gaps are reported "with the existing kinds ... with a message that says 'which strict mode enforces because `cap:<slug>` binds `<path>`'". D§3.8: "eligible = ... the coverage kinds of packages enforced only by strict"; "`apply`: a present strict `Owed` whose key is recorded is not reported".

Code: `coverage::findings` (coverage.rs:611-689) emits plain `Finding`s into `self.findings`; there is no `Owed` and no per-finding origin. If the widening is emitted there (as D§3.3 reads), `debt::apply` in `check` must re-derive which `coverage-item-unbound` of `self.findings.clone()` (model.rs:365) is strict-only, by its message text or by re-checking the subject's package against `scope`: a second decision that can drift from the first.

Consequence: a scope gap (a default finding) absorbed as debt is a silently hidden default finding (§H); or a bound-package gap never matched means every such gap stands forever and adoption (T8) cannot pass.

REWRITE TO: as B1. `strict::check` is the only producer of `Owed { finding, key: DebtKey }`; the widened coverage gaps are produced there and never enter `self.findings`; `debt::apply(owed, record)` consumes `Owed` only. Eligibility is then by construction (the design's own §7.10 claim), and `debt-entry-invalid` for a coverage kind additionally requires the subject's crate not to be in `scope`, so a hand-written row for a default gap is refused rather than ignored.

**B3. Two ways to record orphan-line debt; one defeats the ratchet.**

D§3.8 lists `coverage-lines-over-ceiling` among the debt-eligible finding kinds ("eligible = ... and `coverage-lines-over-ceiling`") and also defines `lines` rows of `crate`/`orphans`. The `findings` row schema is "exactly `kind`, `subject`" with no count.

Consequence: a `findings` row `{kind: coverage-lines-over-ceiling, subject: raf-okf}` has a stable key whatever the count, so once recorded the crate's orphan lines can grow without bound and `apply` reports nothing. The census contract this adopts keys counts separately (`job_copies`, census-debt-ratchet.md:37,50,64) for exactly this reason.

REWRITE TO: `DebtKey` is an enum with two variants, `Finding { kind, subject }` and `Lines { crate, orphans }`; `coverage-lines-over-ceiling` produces only the `Lines` variant; a `findings` row of that kind is `debt-entry-invalid`. State in D§3.8 and in the `Debt` contract that line debt is a count and only a count.

### ADVISORY

**A1. One declaration normalizer for `contract-effect-understated`.** D§3.4 step 3 parses the working-tree item "span text from the index ... as `syn::Item`/`ImplItem`/`TraitItem` by its `SymbolKind`" and the merge-base blob "through `raf_source::parse_source` ... and a visitor". Two parse routes feeding one token comparison must agree exactly (attribute order, `#[doc]` from `///`, generic spacing), and `raf-workplan` has no `syn` dependency (Cargo.toml:14-17; `raf_source::syn` is re-exported at lib.rs:200). A disagreement is a false `touched`, which blocks `complete` under strict with no repair the author can make. REWRITE TO: both sides go through `raf_source::parse_source` of the whole file (working tree from the tree, merge base from `git cat-file`), one visitor collects `(path, declaration tokens)` for every item at every depth, one `declaration_of(&syn::Item)` normalizer, compared per candidate path. Uses `raf_source::syn`; no new dependency.

**A2. Percent ceiling at `counted == 0`.** D§3.9 compares "against `orphan * 100 / counted`". An empty or wholly structural crate divides by zero. REWRITE TO: compare in integers without division, `orphan * 100 * scale <= percent_scaled * counted`, with the decimal's scale taken from its text as the evidence limits do; `counted == 0` is then under every ceiling.

**A3. `Case::strict: bool` is a modal boolean in the fixture builder (D§3.10).** The production API is `Strictness`; the fixture should carry `strictness: Strictness` so the "default neighbour" of each strict case is written as `Strictness::Default` and not as the absence of a flag. Small, but it is the seam the both-directions test will be read through.

**A4. `strict-matrix-missing` and `tests/docs_contract.rs:337 matrix_violations` enforce the same rule once the repository is strict (T8).** D§3.1 keeps both "so the inventory is pinned independently of the record". Two enforcement paths for one fact in one repository is the §O cost; the inventory's state pins stay without this function. REWRITE TO: T8 removes `matrix_violations` and its test (`a_delivered_capability_holds_a_verification_matrix`), leaving the rule to the strict gate `self_workplan` now runs.

**A5. Claim keys `-2`/`-3` by document order (D§3.2).** Inserting a repeated bold lead before another renumbers the later clause: growth and `debt-resolved` at once, and the write refuses the growth, so the author must mark a clause they did not touch. This is a stated change-cost, not a correctness fault; add it to the capability's Constraints beside the digest-key rule for Constraints items, or key repeats by their first distinct following word instead of ordinal.

## Verdict

The decisions the caller asked about (setting location, marker family, widened kinds, OKF debt type with census semantics, `coverage lines` and its T5 dependency, task order) are right. The seam the strict findings sit on is not: B1 and B2 are one fix (strict findings, the widening included, produced only by `strict::check` as `Owed`, with `relate` untouched), and B3 is a type shape. All three are small to correct in the plan and large to correct after T2 and T7 have landed.

VERDICT: NEEDS-REVISION
