# Prior-art inventory: issue #27: raf-source: the registry scan example reads the clock

Evidence: prior-art.md, references.md, brief.json. Coverage, copied:
- issue #27 + 0 comment(s) via github
- link crawl: no `#N` references in the issue or its comments
- forge search on ['raf-source:', 'registry', 'example']: 4 other item(s)
- GhpRsq-Defect-Class trailers: 1 distinct class(es) in the last 200 commits
- prior attempts: 0 commit(s) citing #27 (second parents walked), 0 other branch(es) named for it
- PRs citing #27: 0
- requirement candidates: 4 line(s); 0 comment(s) matched no rule
- referenced artifacts: 2 path(s) (1 on disk), 2 identifier(s), 1 workspace package(s), 0 URL(s)

## Referenced artifacts
| Reference | Resolved | Exists | Lines | Licence header | Symbol ranges |
|-----------|----------|--------|-------|----------------|---------------|
| `crates/raf-source/examples/registry_scan.rs` | repo | yes | 671 | SPDX: MIT OR Apache-2.0 OR MulanPSL-2.0 | `Instant` (use, line 64) 64-64; `Instant` (line 562) 562-562; `sleep` (`thread::sleep`, line 572) 572-572 |
| `examples/` | not here; exists in sibling(s): biocode-2026-09-30, cf-config, llama-cpp-rs-logit-loom, purrassist, purrdf (+2 more) | NO | — | — | — |

Symbol ranges are single grep hits (`grep -n`), so start equals end.

## Prior attempts at this issue
- Commits: none found (0 citing #27, second parents walked)
- Branches: none (the only branch is the current working branch `paudley/27-raf-source-the-registry-scan-example`, named by the brief as `own_branch`, not as a prior attempt)
- PRs: none found

## Linked issues
| # | Relation | State | Title |
|---|----------|-------|-------|
| 21 | related (forge search) | CLOSED | Replace wall-clock assertions in tests with operation or allocation counts |
| 2 | related (forge search) | CLOSED | Shared crates: raf-okf + raf-source |
| 14 | related (forge search) | CLOSED | raf-source: model rustc's visibility of re-exported private glob bindings and of ambiguity through inaccessible globs |
| 1 | related (forge search) | CLOSED | Scaffold workspace, project docs and branding, plus spec-first capability docs |

No `#N` links were crawled from the issue (`link_relations` empty).

## Governing ADRs
- none: the issue cites no ADRs (brief `adrs` empty); NOT FOUND

## Recurrence
- `none`: 7 prior instance(s) (counted from `GhpRsq-Defect-Class` trailers; 1 distinct class in the last 200 commits)

## NEEDS-JUDGMENT
1. `examples/` is marked NO (not here; exists in sibling(s): biocode-2026-09-30, cf-config, llama-cpp-rs-logit-loom, purrassist, purrdf, plus 2 more not named in references.md): the issue names `examples/`, which does not exist in this repo, while the in-repo example is `crates/raf-source/examples/registry_scan.rs`. Question for the planner: is `examples/` a file to create here, or a port source the issue did not name? If a port source, which sibling, and which two additional siblings are meant?
2. Licence comparison for `crates/raf-source/examples/registry_scan.rs` (header MIT OR Apache-2.0 OR MulanPSL-2.0): this repository's own licence row is not present in references.md, so the difference is UNKNOWN, not "none". Question for the planner: does a licence comparison apply, given the referenced file is in this repo and no port source is named?
3. Recurrence (`none`: 7) and the three wall-clock related items (#21 CLOSED) have no linked commits or PRs in the inputs, so no earlier-attempt outcome can be read from them. Question for the planner: does #21 (closed, same clock-assertion theme) bear on the plan, and should its outcome be checked elsewhere since the inputs do not link any commit to it?

## Return
Status: SUCCESS
Inventory: /home/paudley/Active/rust-agentic-foundations/.worktrees/27-raf-source-the-registry-scan-example/.stage/raf-source-the-registry-scan-example/prior-art-assessment.md
Artifacts: 2 (missing 1, elsewhere-only 1, licence differs UNKNOWN)
Prior attempts: commits 0, branches 0, PRs 0
NEEDS-JUDGMENT: 3
