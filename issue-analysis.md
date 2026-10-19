# Requirements ledger: issue #29: Index the coverage product instead of pinning entries x exclusions

Evidence: issue.md, requirements-candidates.md, brief.json (coverage: issue #29 + 0 comment(s) via github; link crawl: no `#N` references in the issue or its comments; forge search on ['Index', 'coverage', 'product']: 1 other item(s); GhpRsq-Defect-Class trailers: 1 distinct class(es) in the last 200 commits; prior attempts: 0 commit(s) citing #29 (second parents walked), 0 other branch(es) named for it; PRs citing #29: 1; requirement candidates: 1 line(s); 0 comment(s) matched no rule; referenced artifacts: 0 path(s) (0 on disk), 1 identifier(s), 0 workspace package(s), 0 URL(s))

## Requirements
| R-id | Kind | Source | Quote (verbatim) | From candidate |
|------|------|--------|------------------|----------------|
| R1 | prose | body L7 | "Build an index so the work grows with entries plus exclusions" | prose |
| R2 | prose | body L7 | "replace the exact product pin with a linear-growth check." | prose |
| R3 | modal | body L9 (B1 line 3) | "`exclusion_for` marks every exclusion that matches an entry, not only the first one it keeps" | B1 |
| R4 | modal | body L9 (B1 line 3) | "An index must still return every matching exclusion for each entry" | B1 |
| R5 | modal | body L9 (B1 line 3) | "the report of unused exclusions must stay the same" | B1 |
| R6 | prose | body L9 (B1 line 3) | "an index that stops at the first match would hide unused exclusions." | B1 |

Kinds: checkbox, modal, list-item, scope-out, dependency, question, decision, prose.
Line numbers are issue.md line numbers. B1 line 3 is the line number within the candidate's body extract.

## Candidate dispositions
| Candidate | Disposition |
|-----------|-------------|
| B1 | kept → R3, R4, R5, R6 |

## Comment coverage
| Comment | Author | Rows / disposition |
|---------|--------|--------------------|
(none: issue #29 has 0 comments; requirements-candidates.md reports "None" for comments that matched no rule)

## Dependencies (from brief.json relations)
- none: link_relations is empty and linked_issues is empty
- #21 related (non-link reference) [CLOSED]: "Replace wall-clock assertions in tests with operation or allocation counts"

## NEEDS-JUDGMENT
1. R2 vs R1: "replace the exact product pin with a linear-growth check." / "Build an index so the work grows with entries plus exclusions": the issue does not say what the linear-growth check asserts or how it measures growth; the meaning depends on that choice.
2. R4 vs R5 vs R3: "An index must still return every matching exclusion for each entry" / "the report of unused exclusions must stay the same": the issue does not say whether the index must reproduce the current exclusion_for output exactly or only the matches used for reporting; the meaning depends on that choice.

## Self-check
Quotes checked: 6. Misses: 0. Comments covered: 0/0.
