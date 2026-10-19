# Prior-art inventory: issue #21: Replace wall-clock assertions in tests with operation or allocation counts

Evidence: prior-art.md, references.md, brief.json. Coverage, copied:
- issue #21 + 1 comment(s) via github
- link crawl: no `#N` references in the issue or its comments
- forge search on ['Replace', 'wall-clock', 'assertions']: 1 other item(s)
- GhpRsq-Defect-Class trailers: 1 distinct class(es) in the last 200 commits
- prior attempts: 0 commit(s) citing #21 (second parents walked), 0 other branch(es) named for it
- PRs citing #21: 0
- requirement candidates: 9 line(s); 1 comment(s) matched no rule
- referenced artifacts: 6 path(s) (5 on disk), 11 identifier(s), 3 workspace package(s), 0 URL(s)

## Referenced artifacts
| Reference | Resolved | Exists | Lines | Licence header | Symbol ranges |
|-----------|----------|--------|-------|----------------|---------------|
| `crates/raf-source/tests/exported_items.rs` | repo | yes | 1669 | MIT OR Apache-2.0 OR MulanPSL-2.0 | not searched |
| `crates/raf-okf/tests/validate_cost.rs` | repo | yes | 165 | MIT OR Apache-2.0 OR MulanPSL-2.0 | not searched |
| `crates/raf-workplan/tests/assessments.rs` | repo | yes | 965 | MIT OR Apache-2.0 OR MulanPSL-2.0 | not searched |
| `crates/raf-workplan/tests/rationale.rs` | repo | yes | 608 | MIT OR Apache-2.0 OR MulanPSL-2.0 | not searched |
| `crates/raf-workplan/src/evidence/tests.rs` | repo | yes | 966 | MIT OR Apache-2.0 OR MulanPSL-2.0 | not searched |
| `exported_items.rs` | repo | NO | — | — | — |

Notes:
- `exported_items.rs` (bare name, no directory) is the only missing reference. Its `resolved` column is "repo" in references.md, but no "not here; exists in sibling(s)" marker is recorded for it.
- No reference is marked "not here; exists in sibling(s)".
- Licence headers are copied from references.md. This repository's licence files are LICENSE-MIT, LICENSE-APACHE and LICENSE-MULAN, with `license = "MIT OR Apache-2.0 OR MulanPSL-2.0"` in Cargo.toml. The five on-disk artifacts carry the same expression.

Artifact history (copied from references.md):
- `crates/raf-source/tests/exported_items.rs`: 1c5a765db, f1603cd70, 79fb59834, e0a75e9e1, 5f5049baa, 0738a45e3, 8844239ea
- `crates/raf-okf/tests/validate_cost.rs`: de0b842d2, 671850b2b
- `crates/raf-workplan/tests/assessments.rs`: 16d3648f8, 6b317fc6b, bb415d37c
- `crates/raf-workplan/tests/rationale.rs`: c54db19d2
- `crates/raf-workplan/src/evidence/tests.rs`: 16d3648f8, 9bacdb380

Identifier hit counts at HEAD (copied): `perf` 16, `tree::disk::tests` 3, `resolve_paths` 1, `manifest_caps` 1, `trait_keys` 3, `module_tree` 4, `exported_items` 17, `prescan_budgets` 2, `symbol_index` 3, `facts` 36, `tree::disk` 4.

## Linked work (named in the request; verified on disk)
- `crates/raf-source/src/work.rs`: EXISTS, 237 lines, SPDX MIT OR Apache-2.0 OR MulanPSL-2.0.
  Recent history: c8e5187 perf(raf-source) ...; f1603cd70 test(raf-source): the listing tests count glob view reads and walk steps instead of reading the clock, and a gate keeps the clock out of the crate; 72bedc4 test(raf-source): facts and long-line positions are bounded by counted work, not by the clock.
- `tests/support/counting.rs`: MISSING at the repository-root path named in the request. The file exists at `crates/raf-source/tests/support/counting.rs` (module `support::counting`, used by `crates/raf-source/tests/manifest_caps.rs`). No symbol ranges searched.
- `tests/repo_gates.rs`: EXISTS, 1128 lines. Gate family `time_in_raf_source_` defined at:
  - line 532: `time_in_raf_source_tracked_files_name_no_clock`
  - line 863: `time_in_raf_source_rejects_a_planted_clock_reading_with_file_and_line`
  - line 876: `time_in_raf_source_rejects_each_spelling_in_the_library_and_the_tests`
  - line 904: `time_in_raf_source_passes_the_clean_neighbour`
  - line 915: `time_in_raf_source_exempts_the_examples_and_other_crates_and_other_files`
  The module doc (line 25) names the `time_in_raf_source_` prefix. Last commit touching the file: fc21ada test: compare the clean neighbour of the clock gate to an empty list.

## Prior attempts at this issue
- Commits: none found (no commit cites #21; brief prior_attempts.commits is empty). The timing-conversion commits above (f1603cd70, 72bedc4, c8e5187) are linked work and do not cite #21 in the brief.
- Branches: none (brief lists no branch named for #21). The current branch `paudley/21-replace-wall-clock-assertions-in-tests` is this issue's own branch; `git branch --list '*21*'` shows only it.
- PRs: none found (PRs citing #21: 0; search coverage recorded as performed).

## Linked issues
| # | Relation | State | Title |
|---|----------|-------|-------|
| 14 | related (forge search) | CLOSED | raf-source: model rustc's visibility of re-exported private glob bindings and of ambiguity through inaccessible globs |

No `#N` link crawl hits. linked_issues and link_relations are empty.

## Governing ADRs
- NOT FOUND (the issue cites no ADRs; brief adrs is empty)

## Recurrence
Counted from GhpRsq-Defect-Class trailers (last 200 commits): `none`: 6 prior instance(s). This is one distinct class, and it is the value `none`, so it does not identify a defect class.

## NEEDS-JUDGMENT
1. Referenced path `exported_items.rs` is NO (references.md, bare name, resolved "repo"; the repo has `crates/raf-source/tests/exported_items.rs`, which is a different path). Question: is this a file to create here, or a shorthand the issue meant for the existing `crates/raf-source/tests/exported_items.rs`?
2. Requested linked-work path `tests/support/counting.rs` is MISSING at that path (prior-art and references do not name it; the file is at `crates/raf-source/tests/support/counting.rs`). Question: does the plan cite the crate-local path as the counting helper, and is the root-level path wrong in the plan?
3. Gate `time_in_raf_source` (tests/repo_gates.rs) is the timing conversion gate that the issue's work would extend or reuse. Question: which of the five existing gate functions above does the plan treat as the baseline, and does the plan change the gate's scope?
4. Recurrence: the only class recorded is `none` (6 instances), which is not a defect class. Question: does the planner treat this as first recorded instance for defect-class purposes?
5. Coverage says 1 comment matched no requirement rule. The inventory cannot tell which comment or why. Question: does the planner need that comment reviewed before it relies on the requirement candidates?
