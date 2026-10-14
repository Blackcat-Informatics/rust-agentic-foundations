# Requirement candidates — issue #7

Lines from the issue body and comments that matched a lexical rule, copied
verbatim with the rule that fired. This is a CANDIDATE list: rules both
over-match (a "should" in passing) and under-match (a requirement stated
without a keyword). The issue-analyzer keeps or drops each row, adds what
the rules missed, and accounts for every silent comment. Code fences are
skipped.

| ID | Source | Line | Rules | Text |
|---|---|---|---|---|
| B1 | body | 7 | checkbox | - [ ] **Capability** (`docs/capabilities/<slug>.md`): |
| B2 | body | 8 | list-under-requirement-heading | - frontmatter `type: Capability`, `id: cap:<slug>`, `title`, `description`, `state` (`considered\|designed\|implementing\|delivered`), `bindings{code,tests,examples}`, `depends`; |
| B3 | body | 9 | modal, list-under-requirement-heading | - required sections Purpose, Contract, Usage, Examples, Constraints, Explanation, Verification |
| B4 | body | 10 | checkbox | - [ ] **Change** (`work/changes/<slug>.md`): `type: Change`, `id: change:<slug>`, `changes`, `affects: [cap:<slug>#<section>]`, `contract_effect: none\|changes`, `items: [{claim, outcome}]`, with outcomes `implemented \| dropped \| moved:change:<slug>`. There is no "deferred", per the no-deferrals rule. |
| B5 | body | 11 | checkbox | - [ ] **PROPOSED blocks** (`> [!PROPOSED change:<slug>]`) inside capability docs, so the branch diff shows the code and the docs it will produce side by side |
| B6 | body | 12 | checkbox | - [ ] **Graph**: capabilities, sections, changes, proposals, code symbols, tests; edges for bindings, depends, affects and links |
| B7 | body | 13 | checkbox | - [ ] Binding resolution through `raf-source` (code symbols, test fns, example targets/doctests) |
| B8 | body | 14 | checkbox | - [ ] **Generated blocks** (`<!-- raf:generated:<kind> -->`): verification matrix, relationship table, catalogue/roadmap index (`docs/capabilities/index.md`); `docs sync` rewrites them deterministically |
| B9 | body | 15 | checkbox | - [ ] `status`: roadmap view (planned / implementing with gaps / delivered) |
| B10 | body | 16 | checkbox, modal | - [ ] `check`: schema and closed keys; links resolve; bindings resolve; generated blocks in sync; no PROPOSED block for a non-existent change; a `delivered` capability has no PROPOSED blocks and no empty required section |
| B11 | body | 17 | checkbox | - [ ] `raf_workplan::Gate::workspace(..).assert_clean()` for dev-dependency use |
| B12 | body | 18 | checkbox | - [ ] IDs are slugs only; `check` rejects issue-number-style references in docs |
| B13 | body | 21 | checkbox | - [ ] Fixture tests for every `check` failure |
| B14 | body | 22 | checkbox | - [ ] `docs sync` is idempotent and byte-deterministic |
| B15 | body | 23 | checkbox | - [ ] This repo's own `docs/` passes `check`, or the remaining gaps are recorded explicitly in the obligations output |
| B16 | body | 24 | checkbox | - [ ] Capability docs reconciled |

## Comments that matched no rule

None.
