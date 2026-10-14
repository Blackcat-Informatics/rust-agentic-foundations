# Requirements ledger: issue #7: raf-workplan model: documents, check, graph, bindings, generated docs

Evidence: issue.md, requirements-candidates.md, requirements.json, brief.json (coverage: issue #7 + 0 comment(s) via github; link crawl: 2 issue(s) within 2 hop(s) of `#N` references; forge search on ['raf-workplan', 'model:', 'documents,']: 2 other item(s); GhpRsq-Defect-Class trailers: 1 distinct class(es) in the last 200 commits; prior attempts: 0 commit(s) citing #7 (second parents walked), 0 other branch(es) named for it; PRs citing #7: 0; requirement candidates: 16 line(s); 0 comment(s) matched no rule; referenced artifacts: 2 path(s) (1 on disk), 8 identifier(s), 1 workspace package(s), 0 URL(s))

Line numbers (L) are `issue.md` line numbers. The candidate file's `line` values are body-relative (issue.md line = candidate line + 7). Quotes are the text after any list marker.

## Requirements
| R-id | Kind | Source | Quote (verbatim) | From candidate |
|------|------|--------|------------------|----------------|
| R1 | prose | Context, L9 | "`raf-workplan` puts the planning essay into practice: **plan changes to the documented system, implement them, reconcile the docs with the result, and let Git keep the previous states.**" | prose |
| R2 | prose | Context, L9 | "Each capability has one permanent OKF document that always states the complete current contract." | prose |
| R3 | prose | Context, L9 | "Readers never apply amendments in their heads." | prose |
| R4 | prose | Context, L11 | "Capabilities delivered: `cap:workplan-document-model`, `cap:workplan-generated-docs`." | prose |
| R5 | checkbox | Scope, L14 | "**Capability** (`docs/capabilities/<slug>.md`):" | B1 |
| R6 | list-item | Scope, L15 | "frontmatter `type: Capability`, `id: cap:<slug>`, `title`, `description`, `state` (`considered|designed|implementing|delivered`), `bindings{code,tests,examples}`, `depends`;" | B2 |
| R7 | list-item | Scope, L16 | "required sections Purpose, Contract, Usage, Examples, Constraints, Explanation, Verification" | B3 |
| R8 | checkbox | Scope, L17 | "**Change** (`work/changes/<slug>.md`): `type: Change`, `id: change:<slug>`, `changes`, `affects: [cap:<slug>#<section>]`, `contract_effect: none|changes`, `items: [{claim, outcome}]`," | B4 |
| R9 | checkbox | Scope, L17 | "with outcomes `implemented | dropped | moved:change:<slug>`." | B4 |
| R10 | scope-out | Scope, L17 | "There is no "deferred", per the no-deferrals rule." | B4 |
| R11 | checkbox | Scope, L18 | "**PROPOSED blocks** (`> [!PROPOSED change:<slug>]`) inside capability docs, so the branch diff shows the code and the docs it will produce side by side" | B5 |
| R12 | checkbox | Scope, L19 | "**Graph**: capabilities, sections, changes, proposals, code symbols, tests; edges for bindings, depends, affects and links" | B6 |
| R13 | checkbox | Scope, L20 | "Binding resolution through `raf-source` (code symbols, test fns, example targets/doctests)" | B7 |
| R14 | checkbox | Scope, L21 | "**Generated blocks** (`<!-- raf:generated:<kind> -->`): verification matrix, relationship table, catalogue/roadmap index (`docs/capabilities/index.md`);" | B8 |
| R15 | checkbox | Scope, L21 | "`docs sync` rewrites them deterministically" | B8 |
| R16 | checkbox | Scope, L22 | "`status`: roadmap view (planned / implementing with gaps / delivered)" | B9 |
| R17 | checkbox | Scope, L23 | "`check`: schema and closed keys" | B10 |
| R18 | checkbox | Scope, L23 | "links resolve" | B10 |
| R19 | checkbox | Scope, L23 | "bindings resolve" | B10 |
| R20 | checkbox | Scope, L23 | "generated blocks in sync" | B10 |
| R21 | checkbox | Scope, L23 | "no PROPOSED block for a non-existent change" | B10 |
| R22 | checkbox | Scope, L23 | "a `delivered` capability has no PROPOSED blocks and no empty required section" | B10 |
| R23 | checkbox | Scope, L24 | "`raf_workplan::Gate::workspace(..).assert_clean()` for dev-dependency use" | B11 |
| R24 | checkbox | Scope, L25 | "IDs are slugs only" | B12 |
| R25 | checkbox | Scope, L25 | "`check` rejects issue-number-style references in docs" | B12 |
| R26 | checkbox | Acceptance criteria, L28 | "Fixture tests for every `check` failure" | B13 |
| R27 | checkbox | Acceptance criteria, L29 | "`docs sync` is idempotent and byte-deterministic" | B14 |
| R28 | checkbox | Acceptance criteria, L30 | "This repo's own `docs/` passes `check`" | B15 |
| R29 | checkbox | Acceptance criteria, L30 | "or the remaining gaps are recorded explicitly in the obligations output" | B15 |
| R30 | checkbox | Acceptance criteria, L31 | "Capability docs reconciled" | B16 |
| R31 | dependency | Depends on, L34 | "- #1" | prose |
| R32 | dependency | Depends on, L35 | "- #2" | prose |

Kinds: checkbox, modal, list-item, scope-out, dependency, question, decision, prose.

## Candidate dispositions
| Candidate | Disposition |
|-----------|-------------|
| B1 | kept → R5 |
| B2 | kept → R6 |
| B3 | kept → R7 |
| B4 | kept → R8, R9, R10 (R10 split out as scope-out) |
| B5 | kept → R11 |
| B6 | kept → R12 |
| B7 | kept → R13 |
| B8 | kept → R14, R15 |
| B9 | kept → R16 |
| B10 | kept → R17, R18, R19, R20, R21, R22 |
| B11 | kept → R23 |
| B12 | kept → R24, R25 |
| B13 | kept → R26 |
| B14 | kept → R27 |
| B15 | kept → R28, R29 |
| B16 | kept → R30 |

Candidates kept: 16. Dropped: 0. Prose additions: 6 (R1, R2, R3, R4 from Context; R31, R32 from Depends on). Candidate-file rows contain no quoted, template, acknowledgement or duplicate text, so none were dropped.

## Comment coverage
Comments covered: 0/0. The issue has 0 comments (`## Comments (0)` is empty), and requirements.json lists no silent comments.

## Dependencies (from brief.json relations)
- #1 mentions [CLOSED] (Scaffold workspace, project docs and branding, plus spec-first capability docs)
- #2 mentions [CLOSED] (Shared crates: raf-okf + raf-source)

Note: the issue body lists both under "Depends on", but brief.json records the relation as `mentions`. See NEEDS-JUDGMENT 9.

## Quote self-check
Method: `grep -cF -- '<quote>' issue.md` for each R-row quote. Every quote is a single line, so the full quote was used.

| R-id | Result | Hits |
|------|--------|------|
| R1 | hit | 1 |
| R2 | hit | 1 |
| R3 | hit | 1 |
| R4 | hit | 1 |
| R5 | hit | 1 |
| R6 | hit | 1 |
| R7 | hit | 1 |
| R8 | hit | 1 |
| R9 | hit | 1 |
| R10 | hit | 1 |
| R11 | hit | 1 |
| R12 | hit | 1 |
| R13 | hit | 1 |
| R14 | hit | 1 |
| R15 | hit | 1 |
| R16 | hit | 1 |
| R17 | hit | 1 |
| R18 | hit | 1 |
| R19 | hit | 1 |
| R20 | hit | 1 |
| R21 | hit | 1 |
| R22 | hit | 1 |
| R23 | hit | 1 |
| R24 | hit | 1 |
| R25 | hit | 1 |
| R26 | hit | 1 |
| R27 | hit | 1 |
| R28 | hit | 1 |
| R29 | hit | 1 |
| R30 | hit | 1 |
| R31 | hit | 1 |
| R32 | hit | 1 |

Quotes checked: 32. Misses: 0.
Quote misses: 0

## NEEDS-JUDGMENT
1. R4 vs R22 and R30: "Capabilities delivered: `cap:workplan-document-model`, `cap:workplan-generated-docs`." / "a `delivered` capability has no PROPOSED blocks and no empty required section" / "Capability docs reconciled": the issue is OPEN and asks for capability docs to be reconciled. Question: are these two capabilities already in state `delivered`, and so is R22 already binding on them?
2. R2 vs R11: "always states the complete current contract" / "**PROPOSED blocks** (`> [!PROPOSED change:<slug>]`) inside capability docs". Question: do PROPOSED blocks count as part of the current contract that a reader sees?
3. R6 vs R16: "`considered|designed|implementing|delivered`" / "(planned / implementing with gaps / delivered)". Question: `planned` (R16) is not a `state` value listed in R6. Is `planned` the same as `considered`, or a separate roadmap label?
4. R13 vs R19: "Binding resolution through `raf-source`" / "bindings resolve". Question: what counts as a resolved binding depends on how `raf-source` resolves bindings. That behaviour is defined outside this issue (raf-source is the subject of #2) and is not in the inputs.
5. R12 vs R18: "edges for bindings, depends, affects and links" / "links resolve". Question: which links are meant (doc links, relation links, or both)? The issue does not say.
6. R14 vs R20: "catalogue/roadmap index (`docs/capabilities/index.md`)" / "`<!-- raf:generated:<kind> -->`". Question: the content of `docs/capabilities/index.md` and the set of valid `<kind>` values are not given here. brief.json coverage reports 2 referenced paths, 1 on disk, and the inputs do not say which path is missing.
7. R1 and R2: "puts the planning essay into practice" / "one permanent OKF document". Question: the planning essay and the OKF profile are referenced but not in the inputs. Their wording governs how R1 and R2 are read.
8. R10: "There is no \"deferred\", per the no-deferrals rule." Question: the no-deferrals rule is cited as the basis for the outcome set but its text is not in the inputs. The ledger records the rule as stated and does not interpret it.
9. R31, R32 vs brief.json: "## Depends on" lists "- #1" and "- #2" / brief.json `link_relations` records both as `mentions`. Question: is #1 and #2 a hard dependency or a mention? The ledger keeps both readings.
10. R26 vs R17–R22, R25: "Fixture tests for every `check` failure". Question: the set of check failures is defined only by the check list. The issue does not say whether that list is exhaustive.
11. R29: "or the remaining gaps are recorded explicitly in the obligations output". Question: this is a choice between passing `check` (R28) and recording gaps. The issue does not define the obligations output, and it does not say which option applies if `check` fails.
12. R23: "`raf_workplan::Gate::workspace(..).assert_clean()`". Question: what the `(..)` arguments are is not specified.
