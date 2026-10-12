# T6 review (6a + 6b), independent

## Scope
tests/docs_contract.rs, docs/architecture/{okf-profile,normal-form}.md, docs/capabilities/census-*.md (9), work/changes/census-*.md (9), against plan rows A8, A9, A11-A13, adv.11, dogfood row, issue-refs row, R25-R27, R30, R32, R35-R37 and the approved raf-census design.

## Evidence
- `cargo test --test docs_contract --locked`: 38 tests, 37 pass; only `repository_documents_satisfy_the_profile` fails, with exactly the 4 accepted README links (okf-profile, source-index, workplan-document-model, workplan-cli). No other violation (so no cycles, dangling depends, bad anchors, orphan markers, process wording or issue tokens in the real tree).
- `cargo clippy --all-targets --locked -- -D warnings`: rc 0. `cargo fmt --check`: rc 0.
- `make issue-refs`: rc 0. `uvx reuse@6.2.0 lint` after `git add -N`: compliant, 53/53.
- `git grep -niE 'defer|follow-?up|future work|TBD|TODO' -- docs work tests`: no output.
- R27 spot greps (normal_form = 1, min_tokens 20, SHIM_MAX_TOKENS 30, MIN_SHIM_TOKENS 6, BLAKE3, 4 kinds; 9 subcommands, exit codes; deny_unknown_fields, [[job]], [[distinct]], [job.ext], LedgerExtension; all 14 detector ids; 6 matcher forms + rule:<id>; Rule signature verbatim, negative fixture; census-debt.json, debt --write; cargo metadata, [census] reference; migrate --old-baseline ... --ledger): all present. All 9 are `state: designed`.
- `grep -nwE 'will|would|shall|should'`: only okf-profile.md:30 ("crates will enforce it once they are implemented") and "would" in explanatory counterfactuals (detectors:117, normal-form:158). No unresolved markers, nothing phrased as implemented/published.

## Test review (tests/docs_contract.rs)
Every plan-row rule has a negative test asserting the specific message (closed keys unknown/missing, type, wrong directory, id/file mismatch, state, bindings keys, missing/out-of-order/empty/unexpected section, dangling depends, 2-cycle with exact cycle text, broken link/anchor/outside-repo, orphan and malformed PROPOSED, considered-without-PROPOSED, open-without-record, delivered rules, affects section/slug, contract_effect, outcome (+ accepted moved:), Summary, generated block, hand index, issue token, process wording). Parser hard-fails: unparseable line, duplicate key, unclosed frontmatter, mixed lists, bad map all return Err and are reported as `unparseable frontmatter`; no fallback. Fixtures are built with concat!, so `make issue-refs` passes.

## Findings

### REQUIRED
1. census-debt-ratchet.md is internally inconsistent. Contract: "`debt --write` refuses to raise a recorded count or add a recorded group". Verification: "After `debt --write` on each of the failing fixtures, the check passes", and the failing fixtures include "new group" and "count above the record", which --write must refuse. Further, the Example says a member-set change makes the old group disappear AND the new group unrecorded; --write would then refuse (adds a group), so shrinking a 3-member group to 2 members can never be recorded. The approved design only says: fails on appear/rise, and on disappear/fall until `debt --write`. Fix: either drop the refusal (write the present debt, with the first-adoption case covered) or state exactly which cases --write accepts (resolved groups, lowered counts, member-set shrink to a subset of a recorded group) and restrict the Verification "each failing fixture" line to those; update work/changes/census-debt-ratchet.md item 3 to match.
2. census-rule-api.md describes a Gate-level refusal ("A gate refuses to run a registered rule unless a negative fixture is registered") but the Gate builder lists no way to register fixtures; the only fixture API shown is a separate `raf_census::fixtures::assert_negative` test helper, and the Usage example runs `.rules(...)` with no fixture. The approved design says "A test enforces this". Verification ("registers a rule with no negative fixture and shows the gate fails") cannot be written against the stated API. Fix: either state that enforcement is a test (the registered-rules test helper), or add the fixture-registration method to the Gate/Rule contract and Usage. Align change record item 3.
3. census-cli.md Usage/table show `migrate --old-baseline ... --ledger old.toml` without an output option, while census-migration.md requires `--out` for the output path (and census-cli Constraints names `migrate --out`). State `--out` in the census-cli table and Usage, and say whether it is required.

### ADVISORY
- okf-profile.md says documents carry "exactly" the listed keys yet mentions an optional base key `resource`; the test rejects `resource` as an unknown key. Drop the sentence or say it is not allowed in RAF types.
- okf-profile.md:30 "will enforce it once they are implemented" is forward-looking (no existence claim, so acceptable); consider present-tense wording.
- Invented but acceptable elaborations: JSON report shapes, debt file shape, `--root/--ledger/--out`, `table:hex-lower/upper` names, `VALUE_ADAPTERS` list, pattern id-uniqueness and "unreferenced pattern fails" (the design only says a pattern CAN be referenced), `raf-census = "1"` dev-dependency version, `detector = "isomorphic"` in the ledger example. None contradicts the design, but VALUE_ADAPTERS, the per-detector meanings (e.g. hex-pair-radix) and the "unreferenced pattern fails staleness" rule are not traceable to the approved design; confirm against the source copies or soften.
- census-migration Constraints: the output has an empty `summary` that the ledger check rejects, but census-ledger.md does not say `summary` must be non-empty. Add that to the ledger contract.
- census-detectors.md: "Detectors that read shipping code only are marked so in their report" is vague and has no verification.
- Test gaps (not in the plan row list): no negative for missing frontmatter in a typed directory, unclosed frontmatter, PROPOSED marker outside a capability, `changes` naming an unknown cap, nested document, or the `AC<n>` label. The real-tree test only asserts at least one Architecture document; asserting Capability and Change counts above zero would catch a loader that silently finds nothing.

## Checks passed (no finding)
Closed key sets and state/body/outcome-optionality/SPDX/PROPOSED/generated syntax in okf-profile match the test; no claim that raf-okf or raf-workplan exist. normal-form.md covers token source, forms, thresholds (20/30/6), fingerprint, version bump, golden tests, compatibility. Change records: valid keys, items claim-only, non-empty Summary, affects sections valid, slugs in changes. depends acyclic (test-verified).

Initial verdict (superseded by the Recheck below): BLOCKED

## Recheck
Evidence: `cargo test --test docs_contract --locked` 46 tests, 45 pass; the only failure is the real-tree test, with exactly the 4 accepted README links. clippy `-D warnings` rc 0, fmt rc 0, `make issue-refs` rc 0, reuse lint compliant, defer/follow-up/TBD/TODO grep over docs work tests empty.

1. Debt ratchet: RESOLVED. Growth (new non-subset group, superset, rising count) always fails and `--write` refuses; shrinkage (resolved, strict subset, falling count) fails until `--write`. The Verification now separates growth fixtures (still fail after `--write`) from shrink fixtures (pass after it), adds the strict-subset fixture, and the change record items match. The earlier contradiction and the unrecordable member-shrink case are gone.
2. Rule fixtures: RESOLVED. `fn fixtures(&self) -> RuleFixtures` is on the trait, Gate runs fixtures and refuses empty negative sets or wrong hits, usage example and change record match. ADVISORY: the trait now has three methods, so it extends the approved design's two-method signature (`id`, `check`, both still stated verbatim). The coordinator's decision accepts this; keep the design plan note consistent when the crate is written.
3. `migrate --out`: RESOLVED. Required and consistent in census-cli.md (table, Usage, Constraints) and census-migration.md (Contract, Usage); omitting it is a usage error.

Advisories touched: ledger `summary` non-empty (resolved); okf-profile `resource` rejected, matching the test (resolved); new negatives added (46 tests) and real-tree test requires a Capability (resolved).

VERDICT: PASS
