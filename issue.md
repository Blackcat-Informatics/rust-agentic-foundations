# Issue #7: raf-workplan model: documents, check, graph, bindings, generated docs

State: OPEN   Repo: Blackcat-Informatics/rust-agentic-foundations   Forge: github
Labels: enhancement, crate:raf-workplan

## Body

## Context
`raf-workplan` puts the planning essay into practice: **plan changes to the documented system, implement them, reconcile the docs with the result, and let Git keep the previous states.** Each capability has one permanent OKF document that always states the complete current contract. Readers never apply amendments in their heads.

Capabilities delivered: `cap:workplan-document-model`, `cap:workplan-generated-docs`.

## Scope
- [ ] **Capability** (`docs/capabilities/<slug>.md`):
  - frontmatter `type: Capability`, `id: cap:<slug>`, `title`, `description`, `state` (`considered|designed|implementing|delivered`), `bindings{code,tests,examples}`, `depends`;
  - required sections Purpose, Contract, Usage, Examples, Constraints, Explanation, Verification
- [ ] **Change** (`work/changes/<slug>.md`): `type: Change`, `id: change:<slug>`, `changes`, `affects: [cap:<slug>#<section>]`, `contract_effect: none|changes`, `items: [{claim, outcome}]`, with outcomes `implemented | dropped | moved:change:<slug>`. There is no "deferred", per the no-deferrals rule.
- [ ] **PROPOSED blocks** (`> [!PROPOSED change:<slug>]`) inside capability docs, so the branch diff shows the code and the docs it will produce side by side
- [ ] **Graph**: capabilities, sections, changes, proposals, code symbols, tests; edges for bindings, depends, affects and links
- [ ] Binding resolution through `raf-source` (code symbols, test fns, example targets/doctests)
- [ ] **Generated blocks** (`<!-- raf:generated:<kind> -->`): verification matrix, relationship table, catalogue/roadmap index (`docs/capabilities/index.md`); `docs sync` rewrites them deterministically
- [ ] `status`: roadmap view (planned / implementing with gaps / delivered)
- [ ] `check`: schema and closed keys; links resolve; bindings resolve; generated blocks in sync; no PROPOSED block for a non-existent change; a `delivered` capability has no PROPOSED blocks and no empty required section
- [ ] `raf_workplan::Gate::workspace(..).assert_clean()` for dev-dependency use
- [ ] IDs are slugs only; `check` rejects issue-number-style references in docs

## Acceptance criteria
- [ ] Fixture tests for every `check` failure
- [ ] `docs sync` is idempotent and byte-deterministic
- [ ] This repo's own `docs/` passes `check`, or the remaining gaps are recorded explicitly in the obligations output
- [ ] Capability docs reconciled

## Depends on
- #1
- #2

## Comments (0)

