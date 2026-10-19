# Prior-art inventory: issue #29: Index the coverage product instead of pinning entries x exclusions

Evidence: prior-art.md, references.md, brief.json. Coverage, copied:
- issue #29 + 0 comment(s) via github
- link crawl: no `#N` references in the issue or its comments
- forge search on ['Index', 'coverage', 'product']: 1 other item(s)
- GhpRsq-Defect-Class trailers: 1 distinct class(es) in the last 200 commits
- prior attempts: 0 commit(s) citing #29 (second parents walked), 0 other branch(es) named for it
- PRs citing #29: 1
- requirement candidates: 1 line(s); 0 comment(s) matched no rule
- referenced artifacts: 0 path(s) (0 on disk), 1 identifier(s), 0 workspace package(s), 0 URL(s)

## Referenced artifacts
| Reference | Resolved | Exists | Lines | Licence header | Symbol ranges |
|-----------|----------|--------|-------|----------------|---------------|
| (no paths named) | n/a | n/a | n/a | n/a | n/a |
| identifier `exclusion_for` | n/a | 1 file at HEAD containing it (file path not recorded in references.md) | n/a | n/a | not located (no file path given; no grep run) |

Workspace packages: none named. URLs: none named.

## Prior attempts at this issue
- Commits: none found (prior-art.md: "Commits citing it: None found"; coverage: 0 commit(s) citing #29)
- Branches: none (own branch `paudley/29-index-the-coverage-product-instead-of` is the current worktree branch, not a prior attempt; brief.json `own_branch`)
- PRs: #30 Replace wall-clock assertions in tests with operation counts and scripted clocks [MERGED] (brief.json prior_attempts.prs, this_stage false; prior-art.md PRs citing #29)

## Linked issues
| # | Relation | State | Title |
|---|----------|-------|-------|
| 21 | related (forge search; brief.json `related`) | CLOSED | Replace wall-clock assertions in tests with operation or allocation counts |

No `#N` links were crawled (brief.json `linked_issues` and `link_relations` are empty).

## Governing ADRs
- NOT FOUND (the issue cites no ADRs; brief.json `adrs` is empty)

## Recurrence
`none`: 8 prior instance(s) (counted from GhpRsq-Defect-Class trailers, brief.json `defect_classes`). Tallies copied as recorded; 1 distinct class in the last 200 commits.

## NEEDS-JUDGMENT
1. Identifier `exclusion_for` has 1 file at HEAD (references.md, Identifiers) but no file path is recorded: the planner needs to decide whether the file is in scope for the index change, which requires locating it (not done here).
2. PR #30 [MERGED] is the only PR citing #29 (prior-art.md, Prior attempts; brief.json prior_attempts.prs). Its title covers wall-clock assertions, which matches related issue #21 [CLOSED] more than the coverage-product indexing named in #29. The planner decides whether #30 is a prior attempt at this issue or a related change, while #29 remains OPEN.
3. The issue names no ADRs, paths, workspace packages, or URLs (references.md, Paths/Workspace/URLs: None). The planner decides whether the lack of named artifacts is intended or means the plan must identify the files itself.
