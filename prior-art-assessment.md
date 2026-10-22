# Prior-art inventory: issue #16: raf-workplan strict mode: force coverage and harder gates

Evidence: prior-art.md, references.md, brief.json. Coverage, copied:
- issue #16 + 0 comment(s) via github
- link crawl: 6 issue(s) within 2 hop(s) of `#N` references
- forge search on ['raf-workplan', 'strict', 'mode:']: 4 other item(s)
- GhpRsq-Defect-Class trailers: 1 distinct class(es) in the last 200 commits
- prior attempts: 0 commit(s) citing #16 (second parents walked), 0 other branch(es) named for it
- PRs citing #16: 0
- requirement candidates: 19 line(s); 0 comment(s) matched no rule
- referenced artifacts: 2 path(s) (2 on disk), 8 identifier(s), 2 workspace package(s), 0 URL(s)

## Referenced artifacts
| Reference | Resolved | Exists | Lines | Licence header | Symbol ranges |
|-----------|----------|--------|-------|----------------|---------------|
| `docs/capabilities/index.md` | repo | yes | 35 | SPDX: CC-BY-4.0 (document, not source) | none (no symbol named) |
| `Cargo.toml` | repo | yes | 56 | SPDX: MIT OR Apache-2.0 OR MulanPSL-2.0 | none (no symbol named) |

Recent history (from references.md):
- `docs/capabilities/index.md`: 10 commits, newest `4dcdebc99 docs(capabilities): deliver the workplan-cli capability again`, oldest listed `92521b0a9 docs(capabilities): deliver the okf-profile capability again`.
- `Cargo.toml`: 8 commits, newest `c0685a4c2 feat(cargo-workplan): check, status and docs commands over raf-workplan`, oldest listed `a4eacdcfd build: scaffold RAF workspace`.

Identifier hit counts at HEAD (files containing each): `delivered` 85, `bindings.examples` 18, `strict` 41, `Strictness` 2, `verify` 47, `dropped` 63, `moved` 113, `KINDS` 18.
Workspace packages named: `cargo-workplan`, `raf-workplan`. URLs: none.

## Prior attempts at this issue
- Commits: none found (prior-art.md "Commits citing it": None found; brief.json `prior_attempts.commits` is empty; coverage: 0 commit(s) citing #16)
- Branches: none (prior-art.md "Other branches named for it": None; brief.json `prior_attempts.branches` is empty). The brief records `own_branch` as `paudley/16-raf-workplan-strict-mode-force-coverage`, which is the branch this stage belongs to, not a prior attempt.
- PRs: none found (prior-art.md "PRs citing it": None found; coverage: PRs citing #16: 0)

## Linked issues
| # | Relation | State | Title |
|---|----------|-------|-------|
| 4 | mentions | OPEN | raf-census ledger: unified schema, staleness, debt ratchet, migration |
| 8 | mentions | CLOSED | raf-workplan change lifecycle + cargo-workplan CLI |
| 10 | mentions | CLOSED | M2: rest of the planning essay (entity history, evidence-claims, prose assessment, two-way coverage, current rationale) |
| 3 | 2-hop | OPEN | raf-census core: normal form v1, fingerprints, grouping, reports |
| 7 | 2-hop | CLOSED | raf-workplan model: documents, check, graph, bindings, generated docs |
| 9 | 2-hop | OPEN | Acceptance scenarios A & B + workplan self-gate |

Related items on the forge (prior-art.md "Related items"; brief.json `related`), not in the crawl above: #1 Scaffold workspace, project docs and branding, plus spec-first capability docs [CLOSED]; #2 Shared crates: raf-okf + raf-source [CLOSED]. #8 and #10 appear in both lists.

## Governing ADRs
- The issue cites no ADRs. (brief.json `adrs`: empty) — NOT FOUND

## Recurrence
Counted from `GhpRsq-Defect-Class` trailers (brief.json `defect_classes`, prior-art.md Recurrence):
- `none`: 8 prior instance(s)

The trailer tally is 8 instances of class `none`. These are counts of trailers, not a count of attempts at this issue.

## NEEDS-JUDGMENT
1. Referenced artifacts: both named paths exist in this repo (`docs/capabilities/index.md`, `Cargo.toml`); none is marked NO, and none is elsewhere-only, so no sibling-only port source is named by the brief. Question: confirm the issue's scope names no further file to create or port, since the referenced set is only these two files and the workspace manifest.
2. Licence: no port or source file is named that carries a licence header differing from this repository's licence. This repo's manifest (`Cargo.toml`) header is `MIT OR Apache-2.0 OR MulanPSL-2.0`; the capability index is a CC-BY-4.0 document. Question: none raised by the facts; the planner decides whether either header bears on the plan.
3. Recurrence: the brief records 8 `none`-class trailers and no class tied to this issue. Question: the trailers do not identify which, if any, prior work targeted this issue, so the planner decides whether `none` tallies are relevant as recurrence evidence for this change.
4. Linked open issues #3, #4 and #9 are OPEN and linked (#4 by mention; #3 and #9 by 2-hop). Question: whether any open linked issue constrains the scope of strict mode or gates here; the inventory records states only.
