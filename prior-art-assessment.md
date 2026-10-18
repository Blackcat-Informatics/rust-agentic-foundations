# Prior-art inventory: issue #14: raf-source: model rustc's visibility of re-exported private glob bindings and of ambiguity through inaccessible globs

Evidence: prior-art.md, references.md, brief.json. Coverage, copied:
- issue #14 + 0 comment(s) via github
- link crawl: no `#N` references in the issue or its comments
- forge search on ['raf-source:', 'model', "rustc's"]: 0 other item(s)
- GhpRsq-Defect-Class trailers: 1 distinct class(es) in the last 200 commits
- prior attempts: 0 commit(s) citing #14 (second parents walked), 0 other branch(es) named for it
- PRs citing #14: 0
- requirement candidates: 4 line(s); 0 comment(s) matched no rule
- referenced artifacts: 0 path(s) (0 on disk), 1 identifier(s), 1 workspace package(s), 0 URL(s)

## Referenced artifacts
The issue names no paths. references.md: "None named." The artifacts below were named by the caller, not by the issue. They are checked here against the worktree, so they are not in the brief's counts.

| Reference | Resolved | Exists | Lines | Licence header | Symbol ranges |
|-----------|----------|--------|-------|----------------|---------------|
| crates/raf-source/src/resolve.rs | this repo (caller-named) | YES | 1415 | SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0 (matches repo) | none (issue names no symbol) |
| crates/raf-source/src/settle.rs | this repo (caller-named) | YES | 2940 | MIT OR Apache-2.0 OR MulanPSL-2.0 (matches repo) | none |
| crates/raf-source/src/glob.rs | this repo (caller-named) | YES | 195 | MIT OR Apache-2.0 OR MulanPSL-2.0 (matches repo) | none |
| crates/raf-source/src/glob_scope.rs | this repo (caller-named) | YES | 973 | MIT OR Apache-2.0 OR MulanPSL-2.0 (matches repo) | none |
| crates/raf-source/tests/use_resolution.rs | this repo (caller-named) | YES | 3197 | MIT OR Apache-2.0 OR MulanPSL-2.0 (matches repo) | none |

Repository licence, from its own licence files: LICENSE-MIT (MIT, Copyright 2026 Blackcat Informatics® Inc.), LICENSE-APACHE (Apache-2.0), LICENSE-MULAN (MulanPSL-2.0). All five headers match this set.

Identifier: `use` appears in 340 files at HEAD (references.md). Workspace package: `raf-source`. URLs: none.

## Prior attempts at this issue
- Commits: none found (brief.json prior_attempts.commits: []; prior-art.md "None found.").
- Branches: none named for #14. The worktree's own branch `paudley/14-raf-source-model-rustc-s-visibility-of` exists and points at main (16ffd2f). `git log main..<branch>` lists 0 commits, so it carries no attempt work.
- PRs: none found (prior-art.md "PRs citing it: None found"; brief.json prs: []).

## Linked issues
| # | Relation | State | Title |
|---|----------|-------|-------|
| — | none referenced (brief.json linked_issues: {}, link_relations: {}, related: []) | — | — |

Related history, not linked by the brief: commit 762fa99 "raf-okf and raf-source: OKF profile reader and Rust source index (#13)" is the commit that introduced the raf-source crate in this history. 31 commits touch `crates/raf-source`. Commits in that range that name the resolver behaviour, cited by subject: d8aba1e "fix(raf-source): validate resolve paths and refuse globs that name nothing"; df71a42 "fix(raf-source): match items_under by whole segment and check binding roles"; be71d40 "fix(raf-source): bound glob scope tables with a byte budget"; 5a5e499 "fix(raf-source): settle import cycles by rustc's rules, independent of order"; 0738a45 "test(raf-source): pin the first carrying glob as the site of a re-export two globs bring in".

Rustc-oracle harness: none found. Searches: `grep -i oracle` over crates/raf-source (no hits); `grep -i rustc` over crates/raf-source/tests lists exported_items.rs, module_tree.rs, load_budgets.rs, trait_keys.rs, symbol_index.rs and use_resolution.rs (text references, not a harness; not inspected further); `git log --all --grep='oracle\|rustc-'` (no hits); `git log --all -S'oracle' -- crates/raf-source` (no hits). The only "oracle" hits in the repo are in crates/raf-workplan (link-checking term) and docs/capabilities/workplan-document-model.md, which are unrelated.

## Governing ADRs
- NOT FOUND (the issue cites no ADRs; brief.json adrs: []).

## Recurrence
Counted from GhpRsq-Defect-Class trailers, as the brief recorded them:
- `none`: 6 prior instance(s)
- 1 distinct class(es) in the last 200 commits.

## NEEDS-JUDGMENT
1. Referenced artifacts: references.md records 0 issue-named paths. The five caller-named files above all exist in this repo, with matching licence headers, so no path is NO, elsewhere-only, or licence-differs. Question for the planner: should the issue body be read as naming these resolver modules and test as its scope, given it names none itself?
2. Own branch: `paudley/14-raf-source-model-rustc-s-visibility-of` has 0 commits ahead of main (16ffd2f), so there is no earlier attempt to assess. Question for the planner: is this a fresh start, with no prior work to reconcile?
3. Rustc-oracle harness: none exists in the working tree or in any branch history (searches above). Question for the planner: does the issue require creating a rustc-oracle harness, and if so where should it live, given the issue names no harness?
4. Linked work outside the brief: the 31 raf-source commits (including 762fa99, #13) are not linked by #14 in the brief. Question for the planner: should #13 or the listed fix commits be cited as prior work for the glob and visibility modelling?
5. Recurrence: the only class recorded is `none` (6 instances), which is a "no defect class" marker rather than a defect class. Question for the planner: does this count as a first recorded instance for the defect-class history, or should it be read as no recurrence data?

Coverage has no UNKNOWN entries, so no UNKNOWN is carried into this inventory.
