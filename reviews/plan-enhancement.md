# Enhancement audit: strict mode, claim coverage, reverse coverage, debt ratchet

Reviewed: `.stage/raf-workplan-strict-mode-force-coverage/plan.md` (Rev 0) with `design.md` (D§n).
Standing constraints: no `.goals` file exists in the worktree or the main checkout; the
constraints read are CLAUDE.md and AGENTS.md (no timing runs, fail hard, closed lists, no
deferrals, no tracker numbers in committed files). No item below conflicts with them, so
nothing is marked REJECTED. No web tool was used.

The plan is thorough and most rules are well-derived. The items below are where it builds a
narrower thing than the domain offers, or leaves a gap a fixture should close. Items 1 and 2
are the transformational ones; 11 and 13 are the robustness gaps that most need a decision.

## TRANSFORMATION

1. **Make item-to-item reachability a product of `raf-source` (a reference graph with the
   bindings as its root set), not a private pass of `coverage/lines.rs`.**
   The issue's "inside an item reachable from one through the source index" is a shadow of
   mark-and-sweep reachability over a reference graph: roots = the items every binding resolves
   to; edges = *contains* (what `Bound::covers` reads as "below"), *impl-of* (an impl block to
   its self type) and *references* (an item's tokens name a path the index resolves). D§3.9
   builds exactly those three edges, but as rules inside the lines report: it tokenizes each
   unit's text, extracts `Ident (:: Ident)*` runs and resolves each with `resolve_from`, memoized
   per module. T5 already opens `raf-source` for target-scoped resolution and `impls()`; the
   token walk and resolution belong there too, as a stated contract of `cap:source-index`:
   `Workspace::references(ItemId) -> &[ItemId]` (or a `ReferenceGraph` with forward and
   reverse edges), computed once per workspace, counted, deterministic, with the same stated
   limits (field-stored functions, trait-object dispatch and macro-only use are not edges).
   Then `coverage lines` is the closure of the roots over the three edge kinds, a few dozen
   lines in raf-workplan, and the SAME product answers the next issues in this area:
   (a) `contract-effect-understated` and every future "which claims does this diff touch"
   are reverse-edge queries from touched items to bound roots to capabilities;
   (b) a provenance query (`coverage lines --item <path>` or `cargo workplan explain <path>`:
   item -> chain -> binding -> capability) is free, and it is what paying down ~10,000 orphan
   lines of recorded debt requires (item 8);
   (c) test selection for `verify` (only the tests whose reach includes a touched item);
   (d) the census (`entry_points`, a dead-helper detector) consumes the same graph instead of
   growing a second token walk.
   Cost: L (it is the same work as D§3.9 rule 3, moved one crate down and made a contract with
   its own fixtures and work tallies). Payoff: decisive for the next three issues; it also
   removes the "estimates ... same-target suffix match" caveat of D§6.3 because resolution and
   reach are one product. Where: T5 ("resolution from every target, impl blocks") gains the
   reference graph; D§3.9 "**Covered.** ... 3. it is **reachable**" and the "**Work.**"
   paragraph become consumers of it; `cap:source-index` gains a Contract clause.

2. **Model `Strictness` as a closed rule set, with `strict` as the preset that selects all.**
   D§2.2's `enum Strictness { Default, Strict }` is the shadow of a policy lattice: the strict
   rules are a closed family (examples, matrix, claims, explained items, completion, sections,
   reasons, untouched, lines). Represent the effective strictness internally as a set over a
   closed `StrictRule` enum (`ALL`, compile-time pinned like `ObligationKind`), with
   `Strictness::Strict` = the full set and the record key unchanged (`strictness: strict`
   only; no per-rule key now, so the profile stays small and closed). "Effective = max" becomes
   set union (a caller can only add). Each `Owed` names its rule, so debt eligibility
   (D§3.8 "eligible = every finding that strict alone raises") is derived by construction
   ("raised by a rule in the strict set") instead of a hand-maintained list, and the
   both-directions test in `tests/kinds.rs` can pin rule <-> kind in both directions. The
   design's own "condition to revisit: strict splits into independently chosen rules" then
   costs one optional key later, not a refactor of every `strictness()` check.
   Cost: S-M. Payoff: closes the eligibility list by construction (fail-hard spirit), and
   future-proofs the setting. Where: D§2.2 `Strictness`, D§3.8 "eligible =", D§3.10 table
   (add a Rule column), `tests/kinds.rs` both-directions test.

## LEVERAGE

3. **Put the declaration normal form in `digest.rs` as `Digest::declaration`, beside
   `Digest::symbol`.** D§3.4 step 3 hand-rolls "tokens with `#[doc]` removed and bodies
   replaced by `{}`" inside `lifecycle/touched.rs` with a visitor into a multiset. The same
   normal form is a general product: `contract-effect-understated` compares per-item
   declaration digests at the merge base and now; evidence records could later take a
   `symbol-declaration` input kind so evidence does not go stale on body-only edits
   (`evidence-changed` today reads the whole span, `digest.rs:82-100`); debt subjects for
   coverage gaps could carry it. Cost: S (same code, placed as a product with its own
   fixtures: doc edit, body edit, attribute edit, cfg twins). Where: D§3.4 step 3, T4.

4. **Have the gate subsume the Scenario B test-side detector, and prove it.** The issue's
   premise is that the expectations are "asserted by a test-side detector, not by the gate"
   (`tests/scenarios.rs:1000-1165`, `Account::faults`: example binding, matrix rows, Usage
   fence, Constraints bullets, API references). The plan never touches the detector. After
   T2/T3, judge the Scenario B end state with `Workplan::with_strictness(Strict).check()`
   (expect empty) and the deliberately thin document of `scenarios.rs:1531-1536` with the
   strict kinds (`strict-section-thin`, `strict-example-missing`, `strict-usage-unbound`,
   `strict-limitations-missing`), keeping in the detector only what strict does not state
   (the API-reference count, the fence compiling). Cost: S. Payoff: one source of truth; a
   drift between detector and gate becomes a failing test. Where: T2/T3 checks; D§1.2 last
   paragraph.

5. **Exclusions imply exceptions.** D§3.9 excepts "a test module whose tests are excluded by
   the coverage record ... with that reason", but only tests. Every `exclusions` row (item or
   `<module>::*` prefix) should except the lines of what it takes out, so an author never
   writes the same reason twice for one item; `exceptions` is then needed only for private
   items, files and directories (which `exclusions` cannot name: an exclusion of a private
   item is `coverage-exclusion-unmatched`). Cost: S. Where: D§3.9 "**Exceptions**"; T6 fixture
   "an excluded public item is not an orphan line".

## UTILITY

6. **A `claim-matrix` generated block, opt-in per document.** The issue asks for "the mapping
   from matrix rows to claims"; D§3.2 puts the fine mapping only in a CLI report and the
   coarse one in the matrix Section cell ("[Contract](#contract)"). Requirements traceability
   earns its value when the matrix is *in* the document: a `<!-- raf:generated:claim-matrix -->`
   block (Claim | Line | Tested by), kept current by `docs sync`/`generated-stale`, absent by
   default so every existing document is byte-identical. Cost: M (a second block type in
   `generated.rs`; the rows come from `claims_report`). Payoff: reviewers and assessors read
   coverage where the claims are; `docs_contract` can require it for the inventory under
   strict. Where: D§3.2 "The claim-level table is a report, not a generated block".

7. **Ceilings by line class.** `ceilings` rows hold `orphan` total; the repository's own
   numbers (raf-source 1,140 shipping vs 4,944 test orphan lines) show the useful policy is
   "shipping lines 0, test support tolerated". Add an optional `of: shipping|test|all`
   (default `all`) to a ceiling row, so adoption can hold shipping lines tight now while test
   support is debt. Cost: S. Where: D§3.9 "**Ceilings**"; T6 fixtures (one per class).

8. **Provenance in the lines report.** With item 1 it is free; without it, still add
   `coverage lines --item <path>` (or `lines-covered` JSON rows with `via`: the binding and
   the chain that reaches the item) so a reader can answer "why is this line covered / which
   bound item is nearest". Paying down the recorded line debt is otherwise guesswork.
   Cost: S with item 1, M without. Where: D§3.9 "**Report.**".

9. **`cargo workplan debt` without `--write` as a report**: present debt vs record, per kind
   and crate (standing / resolved / new), text and JSON, exit 1 on any difference. `check`
   gives the findings, but a reviewer of a 2,400-line debt file wants the summary, and
   dashboards want the JSON. Cost: S (the comparison exists for `apply`). Where: D§2.3 CLI,
   T7.

## ROBUSTNESS

10. **The line debt is a per-crate count, which admits trading.** D§3.8 records
    `lines: [{crate, orphans}]`; removing 100 orphan lines in module A while adding 100 in
    module B passes the ratchet. The census semantics the plan adopts is member *sets*
    (strict subset = shrinkage). Record line debt per orphan item (`item`, `orphans`), or at
    least per module; a new orphan item is then growth whatever the total. Cost: S-M (rows
    grow to the number of orphan items, still well inside the document bound). Where: D§3.8
    `lines:` rows and `apply`, D§3.9 "Debt" column, T7 growth fixture "a new orphan item while
    the total falls fails".

11. **A Contract with no clause start is one claim.** D§3.2: "Text before the first clause
    start, if any, is one clause, the opening." A delivered capability whose Contract is plain
    paragraphs (no bold lead, no `###`) is therefore a single unit, and one marker covers the
    whole Contract: the rule is vacuous for exactly the under-documented capability strict is
    for. Fall back to paragraph grain when a section has no clause start (or when the opening
    exceeds N paragraphs), and fixture it. Cost: S. Where: D§3.2 "**Claim unit.**", T3 unit
    fixtures.

12. **`contract-effect-understated` misses removal and rename of items covered "below".**
    D§3.4 takes candidates from the working tree's surface; a bound type's method deleted or
    renamed under `contract_effect: none` has no candidate, so it passes (a directly bound
    item would be `binding-unresolved`, a covered one is not). Add the reverse direction: a
    merge-base declaration below a bound root with no equal in the working tree is touched.
    Cost: S (the merge-base multiset is already built). Where: D§3.4 step 4; T4 fixtures
    "removed method of a bound type refuses", "renamed method refuses".

13. **Claim keys of Constraints units churn on cosmetic edits.** The `c-<16 hex>` digest folds
    whitespace only; fixing a comma or a capital letter in an uncovered item is "growth" and
    fails the gate until a test is written. Fold case and punctuation too (digest the
    lowercase word sequence), and state in Constraints that a reworded item is a new claim
    while a re-punctuated one is not. Cost: S. Where: D§3.2 "**Claim key**"; D§7 item 4.

14. **CLI coverage of "the record raises, no flag".** T2's CLI checks list `--strict check`
    exit codes; add `cargo workplan check` (no flag) exiting 1 on a record with
    `strictness: strict`, and a `status` line (text and JSON) that prints the effective
    strictness and its source (record, flag, both), so a surprised user sees why. Cost: S.
    Where: T2 check list; D§2.3 "`status` ... the same under it".

15. **Fixture the marker after a heading clause and the Verification section under the word
    rule.** A `###` clause's "statement" is the heading line; the unattached rules of
    `claims.rs:190-212` (after a blank line, after a comment) must be shown to accept a marker
    directly after a heading. And `strict-section-thin` reads prose lines without generated
    blocks, so a Verification section that is a matrix plus one sentence is thin; either say so
    in the rule's fix text or count matrix rows as content. Cost: S. Where: T3 marker fixtures;
    D§3.5 `strict-section-thin`.

## Verdict

Items 1, 2, 10, 11 and 12 change the design's shape or close a hole a strict gate must not
have; the rest are small. The plan is not yet maximal.

VERDICT: NEEDS-REVISION
