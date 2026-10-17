# Requirements ledger: issue #10: M2: rest of the planning essay (entity history, evidence-claims, prose assessment, two-way coverage, current rationale)

Evidence: issue.md, requirements-candidates.md, brief.json (coverage: issue #10 + 0 comment(s) via github; link crawl: 3 issue(s) within 2 hop(s) of `#N` references; forge search on ['planning', 'essay', '(entity']: 0 other item(s); GhpRsq-Defect-Class trailers: 1 distinct class(es) in the last 200 commits; prior attempts: 0 commit(s) citing #10 (second parents walked), 0 other branch(es) named for it; PRs citing #10: 0; requirement candidates: 8 line(s); 0 comment(s) matched no rule; referenced artifacts: 0 path(s) (0 on disk), 3 identifier(s), 0 workspace package(s), 0 URL(s))

Line references below are issue.md line numbers (`cat -n`). The candidate file's "Line" column is not reproduced in issue.md numbering, so candidates are matched by quoted text.

## Requirements
| R-id | Kind | Source | Quote (verbatim) | From candidate |
|------|------|--------|------------------|----------------|
| R1 | checkbox | body L12 | "`cap:workplan-entity-history`: history of a capability and its bindings (`git log --follow`, `-L`), so how a design evolved can be investigated on purpose without loading everyone's working context" | B1 |
| R2 | checkbox | body L13 | "`cap:workplan-evidence-claims`: persisted execution evidence, and which current statements depend on which result; evidence changes invalidate the claims that depend on it" | B2 |
| R3 | checkbox | body L14 | "`cap:workplan-prose-assessment`: recorded human/agent assessments of explanatory prose, invalidated when their inputs change" | B3 |
| R4 | checkbox | body L15 | "`cap:workplan-two-way-coverage`: code→docs (public items no capability explains) and tests→claims, alongside the M1 docs→code and docs→tests checks" | B4 |
| R5 | checkbox | body L16 | "`cap:workplan-current-rationale`: shared architecture pages that state resolved current decisions, plus the \"conditions that would justify revisiting\" structure" | B5 |
| R6 | checkbox | acceptance L19 | "Each capability reaches `delivered` through `cargo workplan change complete`" | B6 |
| R7 | checkbox | acceptance L20 | "Its change record is deleted on completion; the history is in Git" | B7 |
| R8 | checkbox | acceptance L21 | "No deferrals: any scope that leaves an item is `moved:` to a tracked change" | B8 |
| R9 | prose | body Context L9 | "This issue tracks carrying each one through the workplan lifecycle (`reconcile`, then `complete`) using the tool itself." | prose |
| R10 | prose | body Scope heading L11 | "Scope (one item per capability)" | prose |
| R11 | scope-out | body Context L9 | "The rest of the planning essay, beyond milestone 1." | prose |
| R12 | dependency | body Depends on L24 | "- #9" | prose |
| R13 | prose | body Context L9 | "Each item already exists in this repo as a capability document with `state: considered` and a `work/changes/<slug>.md` record (written in the scaffold issue)." | prose |

Kinds: checkbox, modal, list-item, scope-out, dependency, question, decision, prose.

## Candidate dispositions
| Candidate | Disposition |
|-----------|-------------|
| B1 | kept → R1 |
| B2 | kept → R2 |
| B3 | kept → R3 |
| B4 | kept → R4 |
| B5 | kept → R5 |
| B6 | kept → R6 |
| B7 | kept → R7 |
| B8 | kept → R8 |

Candidates kept 8, dropped 0. Prose additions 5 (R9–R13). Note: R10 (scope heading) and R12 (dependency line) were not matched by any lexical rule, so they are additions, not candidates.

## Comment coverage
No comments: brief.json reports 0 comments and issue.md has `## Comments (0)` with no entries. N = 0, so coverage is 0/0 and there are no silent comments to account for. The candidate file's "Comments that matched no rule" is `None.`

## Dependencies (from brief.json relations)
- #9 mentions [state not in brief.json]. Body L24 states "Depends on" #9, so the body and brief.json disagree on the relation kind (see NEEDS-JUDGMENT 1).
- #6 2-hop [state not in brief.json]. Not named in the issue body.
- #8 2-hop [state not in brief.json]. Not named in the issue body.

## NEEDS-JUDGMENT
1. R12 vs brief.json relation: "- #9" (issue.md L24, under "## Depends on") / brief.json `"9": "mentions"`: the issue body declares a dependency on #9, while brief.json classifies #9 only as a mention. Question: is #9 a hard dependency of this issue or only a reference?
2. R4 vs R12: "alongside the M1 docs→code and docs→tests checks" / "- #9": the issue never says which issue delivers the M1 docs→code and docs→tests checks, and brief.json's #9 title is "Acceptance scenarios A & B + workplan self-gate". Question: are the M1 checks that R4 extends delivered by #9, and is that the basis for the dependency?
3. R3 vs R2: "invalidated when their inputs change" / "evidence changes invalidate the claims that depend on it": "their inputs" is not defined in the issue, and it is unclear whether assessment inputs are the same set as evidence results. Question: what counts as an input to a prose assessment?
4. R5: "shared architecture pages" / "the \"conditions that would justify revisiting\" structure": neither "shared" nor the structure's required form is defined in the issue. Question: which pages count as shared, and what form does the conditions structure take?
5. R13 vs brief.json: "(written in the scaffold issue)" (issue.md L9): the change records are said to come from a scaffold issue that is not among brief.json's linked issues (#9, #6, #8), and no referenced path exists on disk per brief.json coverage. Question: which issue is the scaffold issue, and is the premise that each `work/changes/<slug>.md` already exists true?

## Self-check
Quotes checked: 14 (the 13 R-row quotes, with R1 and R5 each counted as one, plus the `## Comments (0)` heading used to confirm the comment section, for 14 greps). Misses: 0. Comments covered: 0/0.
Quote misses: 0
