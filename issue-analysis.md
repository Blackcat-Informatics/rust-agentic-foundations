# Requirements ledger: issue #27: raf-source: the registry scan example reads the clock

Evidence: issue.md, requirements-candidates.md, brief.json (coverage: issue #27 + 0 comment(s) via github; link crawl: no `#N` references in the issue or its comments; forge search on ['raf-source:', 'registry', 'example']: 4 other item(s); GhpRsq-Defect-Class trailers: 1 distinct class(es) in the last 200 commits; prior attempts: 0 commit(s) citing #27 (second parents walked), 0 other branch(es) named for it; PRs citing #27: 0; requirement candidates: 4 line(s); 0 comment(s) matched no rule; referenced artifacts: 2 path(s) (1 on disk), 2 identifier(s), 1 workspace package(s), 0 URL(s))

Line numbers below are issue.md line numbers. The candidate file numbers body lines from the body start, so its line 5 is issue.md line 11.

## Requirements
| R-id | Kind | Source | Quote (verbatim) | From candidate |
|------|------|--------|------------------|----------------|
| R1 | checkbox | body L11 | "- [ ] Replace the per-crate timeout and the sleep poll with a bound on counted work (the index already reports `Usage` and refuses at its budgets), or remove the example if its purpose is served by the differential test against rustc." | B1 |
| R2 | checkbox | body L12 | "- [ ] Extend the gate to cover `examples/` once the example is clean, or state in the gate why it stays exempt." | B2 |
| R3 | checkbox | body L15 | "- [ ] `grep -nE 'std::time|Instant|Duration|sleep|recv_timeout|SystemTime' crates/raf-source/examples` is empty, or the example is gone." | B3 |
| R4 | checkbox | body L16 | "- [ ] The repository gate passes with `examples/` in scope." | B4 |
| R5 | prose | body L8 | "The repository rule is that time-based testing and wall-clock measurement are not allowed" | prose |
| R6 | prose | body L8 | "a repository gate keeps clock spellings out of the crate's source and tests" | prose |

Kinds: checkbox, modal, list-item, scope-out, dependency, question, decision, prose.

## Candidate dispositions
| Candidate | Disposition |
|-----------|-------------|
| B1 | kept → R1 |
| B2 | kept → R2 |
| B3 | kept → R3 |
| B4 | kept → R4 |

Prose additions (step 2): R5 and R6 come from issue.md L8 (Context). No candidate rows were dropped.

## Comment coverage
| Comment | Author | Rows / disposition |
|---------|--------|--------------------|
| (none) | | The issue has 0 comments. Coverage 0/0. |

## Dependencies (from brief.json relations)
- link_relations: none (linked_issues: none)
- related (from brief.json `related`, not dependency edges): #21 related [CLOSED]; #2 related [CLOSED]; #14 related [CLOSED]; #1 related [CLOSED]

## NEEDS-JUDGMENT
1. R1 (internal alternatives): "Replace the per-crate timeout and the sleep poll with a bound on counted work" / "or remove the example if its purpose is served by the differential test against rustc": whether the row is met by a bound on counted work or by removing the example is a choice the issue leaves open.
2. R2 vs R4: "or state in the gate why it stays exempt" (R2) / "The repository gate passes with `examples/` in scope." (R4): R2 allows `examples/` to stay exempt, while R4 requires the gate to pass with `examples/` in scope. Which one governs is unresolved.
3. R2 (internal condition): "once the example is clean" (R2): the extend-or-exempt choice depends on whether the example is cleaned, which depends on R1's choice.
4. R3 vs R1: "is empty, or the example is gone" (R3) / "or remove the example if its purpose is served by the differential test against rustc" (R1): both allow removal of the example as a way to satisfy them. Whether removal alone satisfies R1 and R3 is not stated.
5. R6 vs R2: "a repository gate keeps clock spellings out of the crate's source and tests" (R6) / "Extend the gate to cover `examples/`" (R2): R6 describes the gate as covering the crate's source and tests, and the issue says the gate exempts `examples/`. Whether R6 is in scope for `examples/` is not stated.

## Self-check
Quotes checked: 6. Misses: 0. Comments covered: 0/0.
