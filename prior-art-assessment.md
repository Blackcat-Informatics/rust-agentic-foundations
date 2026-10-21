# Prior-art inventory: issue #3: raf-census core: normal form v1, fingerprints, grouping, reports

Evidence: prior-art.md, references.md, brief.json, references.json, issue.md; plus read-only checks of /home/paudley/Active/purrdf/crates/helper-census, /home/paudley/Active/katamari (shim_key), and the in-repo docs under the worktree. Coverage, copied:
- issue #3 + 0 comment(s) via github
- link crawl: 2 issue(s) within 2 hop(s) of `#N` references
- forge search on ['raf-census', 'core:', 'normal']: 2 other item(s)
- GhpRsq-Defect-Class trailers: 1 distinct class(es) in the last 200 commits
- prior attempts: 0 commit(s) citing #3 (second parents walked), 0 other branch(es) named for it
- PRs citing #3: 0
- requirement candidates: 15 line(s); 0 comment(s) matched no rule
- referenced artifacts: 2 path(s) (2 on disk), 13 identifier(s), 0 workspace package(s), 0 URL(s)

## Referenced artifacts
| Reference | Resolved | Exists | Lines | Licence header | Symbol ranges |
|-----------|----------|--------|-------|----------------|---------------|
| `crates/helper-census` | sibling: /home/paudley/Active/purrdf/crates/helper-census | yes (dir, 22 files) | see table below | SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0 (22 of 22 files); Cargo `license.workspace = true` (crate Cargo.toml line 9), inherited from purrdf root Cargo.toml line 61 `license = "MIT OR Apache-2.0 OR MulanPSL-2.0"` | not grepped (no symbol named by the issue is located in this crate) |
| `docs/architecture/normal-form.md` | repo | yes | 168 | SPDX-License-Identifier: CC-BY-4.0 (line 3) | — |
| `shim_key` (named symbol, katamari, not a referenced path) | /home/paudley/Active/katamari/crates/katamari-helper-census/src/census.rs | yes | — | SPDX-License-Identifier: AGPL-3.0-only (lines 1-2); Cargo.toml line 72 `license = "AGPL-3.0-only"` | census.rs:57-61 |

Repository licence (this repo, root Cargo.toml line 33): `MIT OR Apache-2.0 OR MulanPSL-2.0`. Docs carry CC-BY-4.0.

### helper-census files (sibling, /home/paudley/Active/purrdf/crates/helper-census)
| File | Lines | SPDX licence |
|------|-------|--------------|
| Cargo.toml | 48 | MIT OR Apache-2.0 OR MulanPSL-2.0 |
| examples/glossary_hook_probe.rs | 224 | MIT OR Apache-2.0 OR MulanPSL-2.0 |
| src/asm_reports.rs | 286 | MIT OR Apache-2.0 OR MulanPSL-2.0 |
| src/capacity.rs | 1844 | MIT OR Apache-2.0 OR MulanPSL-2.0 |
| src/census.rs | 849 | MIT OR Apache-2.0 OR MulanPSL-2.0 |
| src/glossary/mod.rs | 1489 | MIT OR Apache-2.0 OR MulanPSL-2.0 |
| src/glossary/pattern.rs | 324 | MIT OR Apache-2.0 OR MulanPSL-2.0 |
| src/glossary/po.rs | 186 | MIT OR Apache-2.0 OR MulanPSL-2.0 |
| src/glossary/surface.rs | 329 | MIT OR Apache-2.0 OR MulanPSL-2.0 |
| src/layout.rs | 846 | MIT OR Apache-2.0 OR MulanPSL-2.0 |
| src/ledger.rs | 480 | MIT OR Apache-2.0 OR MulanPSL-2.0 |
| src/main.rs | 977 | MIT OR Apache-2.0 OR MulanPSL-2.0 |
| src/non_rust.rs | 1002 | MIT OR Apache-2.0 OR MulanPSL-2.0 |
| src/normalize.rs | 1062 | MIT OR Apache-2.0 OR MulanPSL-2.0 |
| src/policy.rs | 670 | MIT OR Apache-2.0 OR MulanPSL-2.0 |
| src/rules.rs | 1232 | MIT OR Apache-2.0 OR MulanPSL-2.0 |
| src/source.rs | 1398 | MIT OR Apache-2.0 OR MulanPSL-2.0 |
| src/structure.rs | 676 | MIT OR Apache-2.0 OR MulanPSL-2.0 |
| src/toml.rs | 522 | MIT OR Apache-2.0 OR MulanPSL-2.0 |
| tests/glossary_callers.rs | 172 | MIT OR Apache-2.0 OR MulanPSL-2.0 |
| tests/non_rust_ratchet.rs | 759 | MIT OR Apache-2.0 OR MulanPSL-2.0 |
| tests/policy_guards.rs | 33 | MIT OR Apache-2.0 OR MulanPSL-2.0 |

Sibling history (references.md, confirmed with git log): latest `b9a014566 Bind assembly aggregation to the measured producer compiler`; 10 commits listed in references.md.

Identifiers named by the issue, at HEAD in this repo (references.md): `blake3` 0 files; `normal_form` 20 files; `shim_key` 2 files; `shim` 11 files; `normalize` 2 files; `proc_macro2` 8 files; `serde_json` 19 files; `min_tokens` 7 files.

### In-repo census docs (worktree)
- docs/architecture/normal-form.md: 168 lines (SPDX CC-BY-4.0). History: 21499bcf6, edf93d5b5, 25e1978a3.
- docs/capabilities/census-normal-form.md: 178 lines (SPDX CC-BY-4.0, id `cap:census-normal-form`). Already states `normal_form = 1` and shim bounds (MIN_SHIM_TOKENS 6, SHIM_MAX_TOKENS 30).
- Other capability docs present: census-cli.md 147, census-debt-ratchet.md 158, census-detectors.md 146, census-ledger.md 211, census-migration.md 193, census-patterns.md 137, census-reference-scan.md 155, census-rule-api.md 200 (total 1525 with normal-form capability 178 included in the same listing).

## Prior attempts at this issue
- Commits: none found (brief.json prior_attempts.commits empty; prior-art.md "None found.")
- Branches: none (other branches named for #3). The worktree's own branch `paudley/3-raf-census-core-normal-form-v1` is recorded in brief.json as own_branch, not as a prior attempt.
- PRs: none found

## Linked issues
| # | Relation | State | Title |
|---|----------|-------|-------|
| 2 | mentions | CLOSED | Shared crates: raf-okf + raf-source |
| 1 | 2-hop | CLOSED | Scaffold workspace, project docs and branding, plus spec-first capability docs |

Issue #3 Depends on: #2 (issue.md).

## Governing ADRs
- The issue cites no ADRs (brief.json `adrs: []`; prior-art.md). NOT FOUND.

## Recurrence
`none`: 8 prior instance(s) (brief.json defect_classes; prior-art.md Recurrence). These are the GhpRsq-Defect-Class `none` trailer tallies from the last 200 commits; they are not counted for this issue.

## NEEDS-JUDGMENT
1. Referenced `crates/helper-census` resolved only in the sibling purrdf checkout (references.md, "sibling"); issue.md Context names purrdf's copy as one of seven diverged copies: the planner decides whether this sibling crate is the port source the plan cites and which of the 22 files (listed above, all MIT OR Apache-2.0 OR MulanPSL-2.0) are in scope.
2. Named symbol `shim_key` is at /home/paudley/Active/katamari/crates/katamari-helper-census/src/census.rs:57-61 (issue.md Scope, shim form). That file's header and Cargo.toml line 72 say AGPL-3.0-only, which differs from this repo's MIT OR Apache-2.0 OR MulanPSL-2.0 (root Cargo.toml line 33). The planner decides the licence and source route for the `shim_key` refinement.
3. `shim_key` also exists in four agent worktree copies under /home/paudley/Active/katamari/.claude/worktrees/ (census.rs:57 and cross.rs:312/317 in each); the planner decides which katamari copy, if any, is the reference.
4. Shim bounds: issue.md says "single-expression forwarders (6–30 tokens)"; the in-repo docs (normal-form.md lines 83-84; census-normal-form.md lines 42-43) say a body of 6 to 29 tokens has a shim form, with MIN_SHIM_TOKENS 6 and SHIM_MAX_TOKENS 30 (fewer than 30). The planner decides which bound the normal form v1 uses.
5. Issue.md expects `docs/architecture/normal-form.md` to be "updated" (168 lines, existing history 21499bcf6, edf93d5b5, 25e1978a3) and `cap:census-normal-form` to be "reconciled" (docs/capabilities/census-normal-form.md, 178 lines, already present): the planner decides how much of the existing text is the baseline.
6. `blake3`: the issue names the `blake3` crate for fingerprints; references.md counts 0 files at HEAD in this repo, and the sibling `normalize.rs:106` uses `purrdf_hash::blake3::hash` (a different path to the same hash). The planner decides which dependency route v1 takes.
7. UNKNOWN coverage: none. All coverage lines in the brief resolved to counts (see Coverage above).

Status: SUCCESS
