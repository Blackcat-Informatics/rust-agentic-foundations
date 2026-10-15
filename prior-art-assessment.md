# Prior-art inventory: issue #8: raf-workplan change lifecycle + cargo-workplan CLI

Evidence: prior-art.md, references.md, brief.json. Coverage, copied:
- issue #8 + 0 comment(s) via github
- link crawl: 3 issue(s) within 2 hop(s) of `#N` references
- forge search on ['raf-workplan', 'change', 'lifecycle']: 1 other item(s)
- GhpRsq-Defect-Class trailers: 1 distinct class(es) in the last 200 commits
- prior attempts: 0 commit(s) citing #8 (second parents walked), 0 other branch(es) named for it
- PRs citing #8: 0
- requirement candidates: 17 line(s); 0 comment(s) matched no rule
- referenced artifacts: 0 path(s) (0 on disk), 7 identifier(s), 1 workspace package(s), 0 URL(s)

## Referenced artifacts
| Reference | Resolved | Exists | Lines | Licence header | Symbol ranges |
|-----------|----------|--------|-------|----------------|---------------|
| (no paths named) | - | - | - | - | - |

Identifiers (files containing each at HEAD, from references.md; no symbol ranges located because no named path exists):
- `reconcile`: 30; `complete`: 38; `delivered`: 46; `clap`: 7; `check`: 130; `status`: 36; `verify`: 21
Workspace package named: `raf-workplan` (references.md; resolution not recorded there).
URLs: none.

## Prior attempts at this issue
- Commits: none found (prior-art.md, "Prior attempts at this issue"; brief.json `prior_attempts.commits` is empty)
- Branches: none (prior-art.md; brief.json `prior_attempts.branches` is empty). The issue's own branch, `paudley/8-raf-workplan-change-lifecycle-cargo`, is recorded in brief.json as `own_branch`; it is not an earlier attempt.
- PRs: none found (prior-art.md, "PRs citing it"; coverage "PRs citing #8: 0")

## Linked issues
| # | Relation | State | Title |
|---|----------|-------|-------|
| 7 | mentions | CLOSED | raf-workplan model: documents, check, graph, bindings, generated docs |
| 1 | 2-hop | CLOSED | Scaffold workspace, project docs and branding, plus spec-first capability docs |
| 2 | 2-hop | CLOSED | Shared crates: raf-okf + raf-source |

Forge search "related items" (prior-art.md): #7 [CLOSED], the same item as above.

## Governing ADRs
- none: the issue cites no ADRs (prior-art.md; brief.json `adrs` is empty)

## Recurrence
Counted from `GhpRsq-Defect-Class` trailers (prior-art.md, brief.json `defect_classes`):
- `none`: 3 prior instance(s)

## NEEDS-JUDGMENT
1. Title and brief.json `title` name a `cargo-workplan` CLI; references.md names only the workspace package `raf-workplan` and no `cargo-*` package or path: the planner decides whether the CLI crate is a new artifact to create, or is covered by the existing `raf-workplan` package.
2. Workspace package `raf-workplan` is named in references.md with no resolution or existence status recorded: the planner decides whether it must be confirmed before planning against it.
3. Linked issue #7 (relation `mentions`, CLOSED, same title as the related forge item) is the only item with a direct relation; #1 and #2 are 2-hop only. No commit or PR cites #8: the planner decides whether #7's delivered work (see its commits in git history, not cited here) is prior art for this issue's scope.
4. Recurrence shows only the class `none` (3 instances); no defect-class is recorded for this issue's own history: the planner decides how to treat this tally, which does not identify a specific defect class.
