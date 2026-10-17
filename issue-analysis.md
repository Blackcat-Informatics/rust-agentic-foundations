# Requirements ledger: issue #9: Acceptance scenarios A & B + workplan self-gate

Evidence: issue.md, requirements-candidates.md, brief.json (coverage: issue #9 + 0 comment(s) via github; link crawl: 4 issue(s) within 2 hop(s) of `#N` references; forge search on ['Acceptance', 'scenarios', 'workplan']: 3 other item(s); GhpRsq-Defect-Class trailers: 1 distinct class(es) in the last 200 commits; prior attempts: 0 commit(s) citing #9 (second parents walked), 0 other branch(es) named for it; PRs citing #9: 0; requirement candidates: 10 line(s); 0 comment(s) matched no rule; referenced artifacts: 1 path(s) (0 on disk), 4 identifier(s), 0 workspace package(s), 0 URL(s))

Line numbers below are issue.md lines. Candidate file line numbers are body-relative (issue.md line = candidate line + 7).

## Requirements
| R-id | Kind | Source | Quote (verbatim) | From candidate |
|------|------|--------|------------------|----------------|
| R1 | checkbox | body L13 (issue.md) | "- [ ] Fixture git repo, built in a temp dir: one capability redesigned 4 times through successive changes (decisions amended and reversed, interfaces and constraints evolving), modelled on the "Amended YYYY-MM-DD" ADR pattern" | B1 |
| R2 | checkbox | body L14 (issue.md) | "- [ ] At the end:" | B2 |
| R3 | list-item | body L15 (issue.md) | "the capability states the active contract directly;" | prose |
| R4 | list-item | body L16 (issue.md) | "no PROPOSED blocks or change files remain on the reading path;" | prose |
| R5 | list-item | body L17 (issue.md) | "all links resolve;" | prose |
| R6 | list-item | body L18 (issue.md) | "the current rationale is retained;" | prose |
| R7 | list-item | body L19 (issue.md) | "every earlier state can be recovered with `git show`." | prose |
| R8 | checkbox | body L22 (issue.md) | "- [ ] The fixture capability starts `considered`, with open questions; the implementation renames the API and drops one guarantee" | B3 |
| R9 | checkbox | body L23 (issue.md) | "- [ ] `complete` refuses until Usage, Examples and the bindings match the real API and the dropped claim has an outcome" | B4 |
| R10 | checkbox | body L24 (issue.md) | "- [ ] It then succeeds, leaving the capability `delivered` with a real usage example, current API references, explicit limitations and a test-traceability table" | B5 |
| R11 | checkbox | body L27 (issue.md) | "- [ ] `tests/self_workplan.rs` runs `raf_workplan::Gate` on this repo, wired into `make check` and the pre-commit hook" | B6 |
| R12 | checkbox | body L28 (issue.md) | "- [ ] Advance this repo's capability states with `cargo workplan status` and `change complete` for every M1 change record" | B7 |
| R13 | checkbox | body L31 (issue.md) | "- [ ] Both scenarios pass as integration tests" | B8 |
| R14 | checkbox | body L32 (issue.md) | "- [ ] Self-workplan and self-census gates both pass in CI" | B9 |
| R15 | checkbox | body L33 (issue.md) | "- [ ] Every M1 capability is `delivered`, with resolved bindings and a generated verification matrix" | B10 |
| R16 | prose | body L9 (issue.md) | "Once they pass, the workplan gate is turned on for this repo" | prose |
| R17 | prose | body L9 (issue.md) | "milestone 1's own change records are completed **with the tool itself**." | prose |
| R18 | dependency | body L36 (issue.md, "## Depends on") | "- #6" | prose |
| R19 | dependency | body L37 (issue.md, "## Depends on") | "- #8" | prose |

Kinds: checkbox, modal, list-item, scope-out, dependency, question, decision, prose.

## Candidate dispositions
| Candidate | Disposition |
|-----------|-------------|
| B1 | kept → R1 |
| B2 | kept → R2 |
| B3 | kept → R8 |
| B4 | kept → R9 |
| B5 | kept → R10 |
| B6 | kept → R11 |
| B7 | kept → R12 |
| B8 | kept → R13 |
| B9 | kept → R14 |
| B10 | kept → R15 |

Dropped: 0. Candidate rows B1-B10 are all checkboxes that state a requirement. The "At the end:" lead-in (B2) is kept because it is not quoted from another source and is not boilerplate.

## Comment coverage
| Comment | Author | Rows / disposition |
|---------|--------|--------------------|
| (none) | - | Issue #9 has 0 comments (brief.json and the issue's "Comments (0)" heading agree). Nothing to cover. |

## Dependencies (from brief.json relations)
- #6 mentions [state not recorded in brief.json]
- #8 mentions [CLOSED]
- #5 2-hop [state not recorded in brief.json]
- #7 2-hop [state not recorded in brief.json]

## NEEDS-JUDGMENT
1. R12 vs R15: "for every M1 change record" / "Every M1 capability is `delivered`": "M1" is not defined anywhere in issue.md. Which capabilities and change records make up "M1" is not stated, and it is unclear whether the two sets are the same. Question: what is the M1 set?
2. R16 vs R11: "Once they pass, the workplan gate is turned on for this repo" / "wired into `make check` and the pre-commit hook": the context sentence ties turning the gate on to both scenarios passing, while R11 does not say whether its wiring waits for R13. Question: must the R11 wiring follow R13, or may it land earlier?
3. R18 and R19 vs brief.json: "- #6" and "- #8" under "## Depends on" / brief.json classifies #6 and #8 as "mentions": the issue lists both as dependencies, while brief.json records only a mention relation. #6's state is not recorded in brief.json. Question: are #6 and #8 hard dependencies of this issue?

## Self-check
Quotes checked: 19. Quote misses: 0. Comments covered: 0/0.
