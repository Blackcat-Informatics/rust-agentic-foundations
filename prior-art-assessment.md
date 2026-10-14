# Prior-art inventory: issue #7: raf-workplan model: documents, check, graph, bindings, generated docs

Evidence: prior-art.md, references.md, references.json, brief.json, issue.md. Coverage, copied:
- issue #7 + 0 comment(s) via github
- link crawl: 2 issue(s) within 2 hop(s) of `#N` references
- forge search on ['raf-workplan', 'model:', 'documents,']: 2 other item(s)
- GhpRsq-Defect-Class trailers: 1 distinct class(es) in the last 200 commits
- prior attempts: 0 commit(s) citing #7 (second parents walked), 0 other branch(es) named for it
- PRs citing #7: 0
- requirement candidates: 16 line(s); 0 comment(s) matched no rule
- referenced artifacts: 2 path(s) (1 on disk), 8 identifier(s), 1 workspace package(s), 0 URL(s)

## Referenced artifacts
Paths from references.md and references.json, then the workplan capability docs and the docs gate named for this inventory. Licence header is the SPDX line read from the file. Symbol ranges: none requested by the issue; not searched.

| Reference | Resolved | Exists | Lines | Licence header | Symbol ranges |
|-----------|----------|--------|-------|----------------|---------------|
| `docs/capabilities/index.md` (issue scope: generated catalogue/roadmap index) | repo | NO | — | — | — |
| `docs/` | repo | yes (dir) | — | — | — |
| `docs/capabilities/workplan-document-model.md` | repo | yes | not counted | CC-BY-4.0 | — |
| `docs/capabilities/workplan-generated-docs.md` | repo | yes | not counted | CC-BY-4.0 | — |
| `docs/capabilities/workplan-cli.md` | repo | yes | not counted | CC-BY-4.0 | — |
| `docs/capabilities/workplan-complete-gate.md` | repo | yes | not counted | CC-BY-4.0 | — |
| `docs/capabilities/workplan-current-rationale.md` | repo | yes | not counted | CC-BY-4.0 | — |
| `docs/capabilities/workplan-entity-history.md` | repo | yes | not counted | CC-BY-4.0 | — |
| `docs/capabilities/workplan-evidence-claims.md` | repo | yes | not counted | CC-BY-4.0 | — |
| `docs/capabilities/workplan-prose-assessment.md` | repo | yes | not counted | CC-BY-4.0 | — |
| `docs/capabilities/workplan-reconcile.md` | repo | yes | not counted | CC-BY-4.0 | — |
| `docs/capabilities/workplan-two-way-coverage.md` | repo | yes | not counted | CC-BY-4.0 | — |
| `docs/capabilities/okf-profile.md` (named by workplan-document-model depends) | repo | yes | not counted | CC-BY-4.0 | — |
| `docs/capabilities/source-index.md` (named by workplan-document-model depends) | repo | yes | not counted | CC-BY-4.0 | — |
| `tests/docs_contract.rs` (existing docs gate) | repo | yes | not counted | MIT OR Apache-2.0 OR MulanPSL-2.0 | — |
| `crates/raf-source/src/lib.rs` (package `raf-source`, named in references.md) | repo | yes | not counted | MIT OR Apache-2.0 OR MulanPSL-2.0 | — |
| `raf-workplan` crate (issue labels `crate:raf-workplan`; `raf_workplan::Gate`) | not named as a path in references.md | NO (no `crates/raf-workplan`; workspace members are `crates/raf-okf`, `crates/raf-source`) | — | — | — |

Last commit touching each in-repo path (`git log -1 --format='%h %s' -- <path>`):
- `docs/`: `a26e75d` build(fuzz) and the ten `docs/` commits listed in references.md (newest `5bcd741 fix(raf-source): index verbatim use groups and bodiless items`)
- `docs/capabilities/index.md`: no history (path absent)
- `docs/capabilities/workplan-document-model.md`: `6c744af docs(capabilities): deliver the OKF profile and source index`
- `docs/capabilities/workplan-generated-docs.md`: `9ce066f docs(capabilities): add workplan capability specs`
- `docs/capabilities/workplan-cli.md`: `6120029 docs(security): state the trust model and harden the designed parsing and subprocess contracts`
- `docs/capabilities/workplan-complete-gate.md`: `edf93d5 docs: resolve contradictions in the census and workplan contracts`
- `docs/capabilities/workplan-current-rationale.md`: `9f93e5d docs(capabilities): add considered workplan capabilities and retire the essay`
- `docs/capabilities/workplan-entity-history.md`: `6120029 docs(security): state the trust model and harden the designed parsing and subprocess contracts`
- `docs/capabilities/workplan-evidence-claims.md`: `f711aa2 docs(workplan): define the seven sections and close the workplan contract gaps`
- `docs/capabilities/workplan-prose-assessment.md`: `f711aa2 docs(workplan): define the seven sections and close the workplan contract gaps`
- `docs/capabilities/workplan-reconcile.md`: `6120029 docs(security): state the trust model and harden the designed parsing and subprocess contracts`
- `docs/capabilities/workplan-two-way-coverage.md`: `f711aa2 docs(workplan): define the seven sections and close the workplan contract gaps`
- `docs/capabilities/okf-profile.md`: `bf08e45 feat(raf-source)!: seal the public types consumers bind to`
- `docs/capabilities/source-index.md`: `5bcd741 fix(raf-source): index verbatim use groups and bodiless items`
- `tests/docs_contract.rs`: `bf08e45 feat(raf-source)!: seal the public types consumers bind to`

Frontmatter state of the two capabilities the issue lists as delivered: `cap:workplan-document-model` is `state: designed`; `cap:workplan-generated-docs` is `state: designed`.

Identifiers (files containing each at HEAD, from references.md): `title` 82, `description` 83, `state` 85, `depends` 53, `changes` 67, `status` 26, `check` 100, `delivered` 30.

## Prior attempts at this issue
- Commits: none found (prior-art.md: 0 commits citing #7, second parents walked; re-checked `git log --grep` for `#7` against the worktree, no body matched)
- Branches: none (prior-art.md: 0 other branches named for it; the only `7-` branch is the own branch `paudley/7-raf-workplan-model-documents-check`, checked out in this worktree)
- PRs: none found (0 PRs citing #7 per prior-art.md; search was done, not unavailable)

## Linked issues
| # | Relation | State | Title |
|---|----------|-------|-------|
| 1 | mentions (brief); issue body lists it under "Depends on" | CLOSED | Scaffold workspace, project docs and branding, plus spec-first capability docs |
| 2 | mentions (brief); issue body lists it under "Depends on" | CLOSED | Shared crates: raf-okf + raf-source |

The 2-hop crawl found no further linked issues. The forge search returned the same two items.

## Governing ADRs
- none: the issue cites no ADRs (prior-art.md, brief.json `adrs: []`)

## Recurrence
Counted `GhpRsq-Defect-Class` trailers (prior-art.md, brief.json):
- `none`: 2 prior instance(s)
- Trailer count re-checked over the last 200 commits: `GhpRsq-Defect-Class: none` occurs 2 times; no other class.

## NEEDS-JUDGMENT
1. Referenced paths not on disk, `docs/capabilities/index.md` (references.md: NO; issue scope names it as the generated catalogue/roadmap index): the planner decides whether this is a file to create in this repo. No sibling-only paths were reported (references.md shows no "not here; exists in sibling(s)" rows).
2. Crate `raf-workplan` (issue.md title, labels `crate:raf-workplan`, scope `raf_workplan::Gate::workspace(..).assert_clean()`): no `crates/raf-workplan` exists and `Cargo.toml` members are `crates/raf-okf` and `crates/raf-source` only. The planner decides whether this is a new workspace member to create. This path is not listed in references.md, so the brief did not resolve it.
3. Overlap between the existing gate and the planned `check`: `tests/docs_contract.rs` already defines `check`, `analyse`, `check_links`, `check_capability_state`, `check_cross_references`, `check_directories`, `check_bindings`, `check_inventory`, `find_cycles`, and fixture tests such as `rejects_missing_section`, `rejects_depends_two_cycle`, `rejects_broken_relative_link`, `rejects_orphan_proposed_marker`, `rejects_delivered_with_proposed_block`, `rejects_affects_slug_not_in_changes`, `rejects_unclosed_generated_block`, `rejects_hand_written_capability_index`, `rejects_issue_number_token`. The planned `check` list (schema and closed keys, links, bindings, generated blocks in sync, no orphan PROPOSED, delivered without PROPOSED or empty sections, no issue-number references) overlaps these. The planner decides whether `raf-workplan check` reuses, replaces, or duplicates `docs_contract.rs`, and whether `make check` and the pre-commit hook (Makefile lines 9-10) keep running it. The test name `rejects_unclosed_generated_block` shows generated-block parsing exists; no test name shows a "generated blocks in sync" or idempotent `docs sync` check.
4. Relation type: the brief lists #1 and #2 as "mentions", but the issue body says "Depends on #1, #2". The planner decides whether these are dependencies for planning purposes.
5. Capability state: the issue says `cap:workplan-document-model` and `cap:workplan-generated-docs` are "delivered", but both have `state: designed` in their frontmatter (see the artifacts section). The planner decides which is correct and whether the reconcile step changes the state.
6. Licence: the workplan capability docs and `okf-profile.md`/`source-index.md` carry `CC-BY-4.0`, while `tests/docs_contract.rs` and `crates/raf-source/src/lib.rs` carry `MIT OR Apache-2.0 OR MulanPSL-2.0`. The repository's root licence files are LICENSE-MIT, LICENSE-APACHE and LICENSE-MULAN, and `LICENSES/CC-BY-4.0.txt` is present. The planner decides whether the new `raf-workplan` code and any new docs use these licences. No port source is named in the issue, so there is no port-viability question to record.
7. Acceptance criteria "This repo's own `docs/` passes `check`, or the remaining gaps are recorded explicitly in the obligations output": the existing `repository_documents_satisfy_the_profile` test already runs over `docs/`. The planner decides whether the gate's failing state, if any, is recorded or fixed.
