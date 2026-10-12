# T7 review: workplan M1 capability docs, change records, okf-profile architecture

## Scope
docs/capabilities/{okf-profile,source-index,workplan-document-model,workplan-generated-docs,workplan-reconcile,workplan-complete-gate,workplan-cli}.md,
the 7 matching work/changes/*.md, docs/architecture/okf-profile.md (read for consistency). No source files modified.
Contract: plan Task 7 (A8, A11, A12, A18, adv.11), R28, R30, R31, R32, R35-R37, approved raf-workplan design, essay at 720262d.

## Evidence
- `cargo test --test docs_contract --locked`: 46 passed, 0 failed, rc=0 (repository_documents_satisfy_the_profile ok).
- R28 state/type/id/changes/items/affects loop over the 7 slugs: no output. State = designed for all 7. No real `> [!PROPOSED` blocks outside code fences (the two hits in workplan-document-model are inside fenced examples).
- Outcomes grep (`implemented|dropped|moved:change:*` only): no stray values (items have no outcome, as A11 allows).
- R32: all depends targets exist; tsort rc=0 (acyclic).
- R35 unresolved-marker grep: only prose uses of "open questions" describing the considered state (model table, Scenario B); not unresolved markers.
- R36 issue-number grep and process-wording grep over docs/work: no hits. `make issue-refs` rc=0.
- R37 `defer|follow-up|future work` over docs/work/README: no hits.
- `git add -N docs work; uvx reuse@6.2.0 lint`: compliant, 67/67 files, rc=0.
- README's six capability links (okf-profile, source-index, census-normal-form, census-cli, workplan-document-model, workplan-cli) all resolve; relative-link loop over README/docs/work printed nothing.
- Key phrases (git grep -il in docs): present for Plan changes to the documented system, amendment, permanent, roadmap, requires reconciliation, obligation, contract_effect, Grounding, Scenario A, Scenario B. Absent (section 5, Task 8 owns): `historical deliberation`, `conditions to revisit`, `git log --follow`; `current rationale` only as one phrase in document-model.

## Design match (all confirmed)
States (4), Change keys, outcomes `implemented|dropped|moved:change:<slug>`, five completion conditions, record deletion as last/only write, obligation kinds (design's three plus item-without-outcome, generated-stale, tense-mismatch), generated kinds (verification-matrix, relationships, catalogue), CLI subcommands incl. docs preview/status/verify, exit codes 0/1/2, source-index Disk/Memory trees, mod-tree, `#[path]` escape rejection (`..` and absolute, file not read), cfg(test) tag, use-alias index. okf-profile capability links to the architecture page and states only crate behaviour. Nothing claims implementation (bindings empty, all designed). Verification sections are concrete; Scenario A/B are in complete-gate and referenced from reconcile. Change records are valid and use slugs only.

## Essay coverage (sections 1-4, 6, 7)
1 present (document-model and reconcile Explanation). Minor gap: "normal links between current topics remain useful" is not stated.
2 present (states table, "being revised" paragraph, roadmap in cli/generated-docs, branch-diff in PROPOSED text).
3 present (seven sections in-shape, promotion needs reconciliation). Minor gap: the essay's mapping of design material to resolved outcome and per-section meaning are only implicit.
4 present (affected docs, command set, small-change rule, mechanical vs explanatory).
6 present (graph, directions, planned-stays-planned, Grounding). The code->docs, tests->claims and evidence->claims questions are only partly stated (4 of 6 directions in document-model).
7 present in full.
Rule-breaking text rewritten: no CHANGE-n, no "deferred", no conversational voice.
For the next task (section 5 and M2): current rationale vs historical deliberation table, "conditions to revisit", shared architecture page pattern, entity-aware history, evidence->claims, tests->docs and code->docs, prose assessment.

## Findings

REQUIRED
1. workplan-document-model.md "Binding resolution" says a capability in `implementing` or `delivered` binds at least one code path and one test. That contradicts workplan-cli (status example `implementing ... gap: bindings.tests is empty`, "lists for each implementing capability the bindings still missing"), workplan-generated-docs (implementing row shows empty bindings as gaps), workplan-complete-gate ("implementing keeps its gaps visible") and the approved design (only `delivered` is constrained). With the rule as written, `check` would reject exactly the gaps `status` and the catalogue display. Restrict the at-least-one rule to `delivered` (and state that `implementing` may have empty or unresolved bindings, which are reported as gaps; a binding that is listed must still resolve).
2. workplan-complete-gate.md "Documentation obligations by effect": a `contract_effect: none` change "requires only the other four conditions". The five conditions include the empty obligation list, and reconcile says only `section-untouched` for contract/usage/examples is relaxed, all other kinds still apply. Reword to: the five conditions are unchanged; the obligation list omits the contract, usage and examples section-untouched obligations.
3. docs/architecture/okf-profile.md says `docs/capabilities` "holds only Capability documents and has no hand-written index", while the capability doc and generated-docs say `docs/capabilities/index.md` is a generated, non-Capability file living in that directory. Reword to name the generated index as the single exception (accepted only when it holds a generated block, never parsed as a Capability).

ADVISORY
- workplan-cli.md Examples: the status sample names real capabilities (census-ledger, workplan-cli) as `delivered`; use neutral example names so it cannot read as a current status.
- "will read" in workplan-complete-gate.md:68 and workplan-cli.md:35 and "will produce" in document-model are command semantics, acceptable under R35, but could be phrased "reads".
- The `tense-mismatch`, `item-without-outcome` and `generated-stale` obligation kinds, and the delivered-state bindings rule in the gate, extend the approved design; harmless and consistent, noted for the design owner.
- Add the one-line "normal links between current topics remain useful" and the essay's design-material to outcome mapping to document-model Explanation if desired.
- R37 expected at most one hit (deferral is not an outcome); there are none, fine, but a one-sentence statement in document-model would document it.

Initial verdict (superseded by the Recheck below): BLOCKED

## Recheck
- `cargo test --test docs_contract --locked`: 46 passed, rc=0. `make issue-refs`: rc=0. `reuse lint` (after `git add -N docs work`): compliant.
- Finding 1 fixed: document-model binding minimums apply only to `delivered`; `implementing` may have empty or unresolved bindings reported as gaps; consistent with cli, generated-docs and gate.
- Finding 2 fixed: gate states all five conditions apply to every change and only the contract/usage/examples `section-untouched` obligations are not raised, matching reconcile.
- Finding 3 fixed: architecture page names the generated `docs/capabilities/index.md` as the single exception.
- Deferral, issue-number and real PROPOSED-block greps remain clean.

VERDICT: PASS
