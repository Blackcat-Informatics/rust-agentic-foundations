# Requirements ledger: issue #14: raf-source: model rustc's visibility of re-exported private glob bindings and of ambiguity through inaccessible globs

Evidence: issue.md, requirements-candidates.md, brief.json (coverage: issue #14 + 0 comment(s) via github; link crawl: no `#N` references in the issue or its comments; forge search on ['raf-source:', 'model', "rustc's"]: 0 other item(s); GhpRsq-Defect-Class trailers: 1 distinct class(es) in the last 200 commits; prior attempts: 0 commit(s) citing #14 (second parents walked), 0 other branch(es) named for it; PRs citing #14: 0; requirement candidates: 4 line(s); 0 comment(s) matched no rule; referenced artifacts: 0 path(s) (0 on disk), 1 identifier(s), 1 workspace package(s), 0 URL(s))

## Requirements
| R-id | Kind | Source | Quote (verbatim) | From candidate |
|------|------|--------|------------------|----------------|
| R1 | list-item | body, Scope | "- Model both rules in the settling of `use` paths, keeping resolution independent of declaration order." | B1 |
| R2 | list-item | body, Scope | "- Add an every-order fixture for each rule." | B2 |
| R3 | list-item | body, Scope | "- Re-run the rustc-oracle fuzz with private globs, with no new disagreements." | B3 |
| R4 | list-item | body, Scope | "- Remove the two sentences from the Contract's Constraints." | B4 |
| R5 | prose | body, Context item 1 | "rustc keeps it no more visible than the binding it re-exports" | prose |
| R6 | prose | body, Context item 2 | "rustc refuses the path, usually as E0659 and sometimes as E0573" | prose |
| R7 | prose | body, Context | "Two further rustc visibility rules are not modelled yet" | prose |

Kinds: checkbox, modal, list-item, scope-out, dependency, question, decision, prose.

## Candidate dispositions
| Candidate | Disposition |
|-----------|-------------|
| B1 | kept → R1 |
| B2 | kept → R2 |
| B3 | kept → R3 |
| B4 | kept → R4 |

## Comment coverage
| Comment | Author | Rows / disposition |
|---------|--------|--------------------|
| (none) | - | issue.md states "Comments (0)"; no comments to cover. Candidates file: "Comments that matched no rule: None." |

## Dependencies (from brief.json relations)
- none: link_relations and related are empty; linked_issues is empty; no `#N` references.
- Non-issue reference: the `cap:source-index` Contract (named in body Context) is not in brief.json and was not fetched.

## NEEDS-JUDGMENT
1. R3 vs the repository: "Re-run the rustc-oracle fuzz with private globs" names a harness, but no rustc-oracle fuzz harness was found in the repository (a grep for "oracle" hits only unrelated raf-workplan files). Question: where does the harness live, and is it in scope to create it under this issue?
2. R6 vs R1/R2: "rustc refuses the path, usually as E0659 and sometimes as E0573" / "Add an every-order fixture for each rule.": the issue does not say whether the ambiguity fixture must assert E0659, E0573, or both, nor whether the index must refuse with a specific code. Question: which error outcome does the every-order fixture assert?
3. R4 vs the Contract: "Remove the two sentences from the Contract's Constraints." / "the `cap:source-index` Contract lists them under Constraints": the Contract text is not in the stage inputs, so the two sentences cannot be matched against R5 and R6. Question: which Contract document holds the sentences, and do they match R5 and R6 in substance?

## Self-check
Quotes checked: 7. Misses: 0. Comments covered: 0/0.
