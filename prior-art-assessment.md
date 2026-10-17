# Prior-art inventory: issue #9: Acceptance scenarios A & B + workplan self-gate

Evidence: prior-art.md, references.md, brief.json. Coverage, copied:
- issue #9 + 0 comment(s) via github
- link crawl: 4 issue(s) within 2 hop(s) of `#N` references
- forge search on ['Acceptance', 'scenarios', 'workplan']: 3 other item(s)
- GhpRsq-Defect-Class trailers: 1 distinct class(es) in the last 200 commits
- prior attempts: 0 commit(s) citing #9 (second parents walked), 0 other branch(es) named for it
- PRs citing #9: 0
- requirement candidates: 10 line(s); 0 comment(s) matched no rule
- referenced artifacts: 1 path(s) (0 on disk), 4 identifier(s), 0 workspace package(s), 0 URL(s)

## Referenced artifacts
| Reference | Resolved | Exists | Lines | Licence header | Symbol ranges |
|-----------|----------|--------|-------|----------------|---------------|
| `tests/self_workplan.rs` | repo | NO | — | — | — |

Identifiers (references.md, files containing each at HEAD): `considered` 51, `complete` 70, `delivered` 81, `raf_workplan::Gate` 15. No workspace packages, no URLs.

Caller-stated, verified on disk in this worktree (not a forge lookup):
- `crates/raf-workplan/tests/scenarios.rs` exists (66698 bytes). `Gate::workspace` is called at lines 589, 601, 619 and 698.
- `tests/docs_contract.rs` exists at repo root (1520 lines). Its header says `Gate::workspace` runs the same checks from disk; `Gate::workspace(gate_root()).assert_clean()` is at line 587. It was last changed in 4dcdebc.

## Prior attempts at this issue
- Commits: none found (prior-art.md, Prior attempts; brief.json prior_attempts.commits = [])
- Branches: none (prior-art.md, "Other branches named for it: None.")
- PRs: none found (prior-art.md, "PRs citing it: None found."; coverage "PRs citing #9: 0")

## Linked issues
| # | Relation | State | Title |
|---|----------|-------|-------|
| 6 | mentions | OPEN | cargo-census CLI, census self-gate, read-only trial vs purrdf/katamari |
| 8 | mentions | CLOSED | raf-workplan change lifecycle + cargo-workplan CLI |
| 5 | 2-hop | OPEN | raf-census detectors and extensibility: built-ins, patterns, Rule trait, Gate, reference scan |
| 7 | 2-hop | CLOSED | raf-workplan model: documents, check, graph, bindings, generated docs |

Related items on the forge (not link-crawled):
| # | State | Title |
|---|-------|-------|
| 10 | CLOSED | M2: rest of the planning essay (entity history, evidence-claims, prose assessment, two-way coverage, current rationale) |
| 8 | CLOSED | raf-workplan change lifecycle + cargo-workplan CLI |
| 1 | CLOSED | Scaffold workspace, project docs and branding, plus spec-first capability docs |

Caller-stated: issue 8 delivered `scenarios.rs` and `docs_contract.rs` (see the verified artifacts above).

## Governing ADRs
- The issue cites no ADRs (prior-art.md; brief.json `adrs` = []). Nothing to look up.

## Recurrence
Counted from `GhpRsq-Defect-Class` trailers (prior-art.md, Recurrence):
- `none`: 5 prior instance(s)
- 1 distinct defect class in the last 200 commits (coverage line)

## NEEDS-JUDGMENT
1. `tests/self_workplan.rs` is named by the issue. It is NO in this repo (references.md, Paths) and no sibling checkout is named or matched, so there is no elsewhere-only source. Question: is this a file to create at that path, or should the reference be dropped or rewritten in the plan?
2. The caller says issue 8 delivered `crates/raf-workplan/tests/scenarios.rs` and `tests/docs_contract.rs`, and `docs_contract.rs` already runs `Gate::workspace` (verified above). `scenarios.rs` is 66698 bytes and already calls `Gate::workspace` in four places. Question: do acceptance scenarios A & B extend this existing file and gate, or are they already covered, so the self-gate is only a new assertion? The plan should say which.
3. Prior attempts: none in any category, so nothing to judge on outcome. `origin/paudley/10-m2-rest-of-the-planning-essay-entity` is a branch for issue 10, not named for #9 in brief.json, so it is not cited here. No question raised.
4. Coverage has no UNKNOWN entries. Nothing to flag.
