# Requirements ledger: issue #16: raf-workplan strict mode: force coverage and harder gates

Evidence: issue.md, requirements-candidates.md, brief.json (coverage: issue #16 + 0 comment(s) via github; link crawl: 6 issue(s) within 2 hop(s) of `#N` references; forge search on ['raf-workplan', 'strict', 'mode:']: 4 other item(s); GhpRsq-Defect-Class trailers: 1 distinct class(es) in the last 200 commits; prior attempts: 0 commit(s) citing #16 (second parents walked), 0 other branch(es) named for it; PRs citing #16: 0; requirement candidates: 19 line(s); 0 comment(s) matched no rule; referenced artifacts: 2 path(s) (2 on disk), 8 identifier(s), 2 workspace package(s), 0 URL(s))

## Requirements
| R-id | Kind | Source | Quote (verbatim) | From candidate |
|------|------|--------|------------------|----------------|
| R1 | checkbox | body, ## Scope | "A `strict` setting. Library: `Gate::strict()` or a `Strictness` option. CLI: a `--strict` flag plus a repo-level setting (for example a field in a config file next to `docs/capabilities/index.md`, or `Cargo.toml` metadata). Propose one location and justify it in the capability document." | B1 |
| R2 | checkbox | body, ## Scope | "Every `delivered` capability binds at least one runnable example or doc test and has a generated verification-matrix block." | B2 |
| R3 | checkbox | body, ## Scope | "Coverage: every Contract and Constraints claim is covered by a bound test or example. Define the claim unit (a bullet, a bold-lead paragraph, or an explicit claim marker) and the mapping from matrix rows to claims. Report unsupported claims as findings with fix text." | B3 |
| R4 | checkbox | body, ## Scope | "Every public item of a bound crate path is explained by some capability (orphan-symbol finding), reusing the two-way coverage direction." | B4 |
| R5 | checkbox | body, ## Scope | "`change complete` additionally requires `verify` to pass for ALL bound items of the listed capabilities and zero tense-mismatch findings on those capabilities, and forbids `contract_effect: none` for changes that touch a bound public item." | B5 |
| R6 | checkbox | body, ## Scope | "Minimum content rules per section for delivered capabilities, reported as findings: non-trivial length, a runnable Usage example, a Limitations statement." | B6 |
| R7 | checkbox | body, ## Scope | "No `dropped` or `moved` item outcome without a stated reason field." | B7 |
| R8 | checkbox | body, ## Scope | "Stricter obligations: `section-untouched` also applies to Usage and Examples for `contract_effect: none`." | B8 |
| R9 | checkbox | body, ## Scope | "A baseline/ratchet option so adopting strict on an existing repository can start from today's findings and fail only on new ones. Coordinate with the census debt-ratchet design (#4) instead of inventing a second ledger." | B9 |
| R10 | checkbox | body, ## Scope | "Reverse coverage mode (`cargo workplan coverage --reverse`, also enforced under strict with a threshold): report the source lines of a crate that map to no deliverable or specification, for example "crate x has 100 lines that no capability binding or contract explains"." | B10 |
| R11 | checkbox | body, ## Scope | "Reverse coverage gate: a configurable ceiling (maximum orphan lines or percentage per crate), a way to mark deliberate exceptions with a reason (generated code, test support, vendored code), and the baseline/ratchet option above so the count can only go down." | B11 |
| R12 | checkbox | body, ## Scope | "Reverse coverage fixtures: a crate with a fully bound public surface reports zero orphan lines; an unbound private helper reports exactly its line range; a bound item covers its nested items and impl blocks; macro-generated items and `#[cfg(test)]` modules are handled and documented; the report is identical across runs and platforms." | B12 |
| R13 | checkbox | body, ## Acceptance criteria | "Each strict rule has a fixture that triggers it and a passing neighbour." | B13 |
| R14 | checkbox | body, ## Acceptance criteria | "Default (non-strict) behaviour and every existing fixture are unchanged." | B14 |
| R15 | checkbox | body, ## Acceptance criteria | "Strict findings carry the same `Finding {kind, file, location, message, fix}` shape, with the new kinds added to the closed `KINDS` list and covered by the both-directions coverage test." | B15 |
| R16 | checkbox | body, ## Acceptance criteria | "CLI integration tests cover `--strict` exit codes 0, 1 and 2." | B16 |
| R17 | checkbox | body, ## Acceptance criteria | "Documented in the capability documents as a new designed capability (for example `cap:workplan-strict`), following the capability-document workflow in AGENTS.md." | B17 |
| R18 | checkbox | body, ## Acceptance criteria | "Once the rules pass, the repository enables strict on its own docs." | B18 |
| R19 | dependency | body, ## Depends on | "Overlaps with #10: the two-way coverage and evidence-claims items there cover code-to-docs and tests-to-claims, which rules 3 and 4 above enforce under strict. Scope should be reconciled between the two so neither duplicates the other." | B19 |
| R20 | dependency | body, ## Depends on | "Builds on #8 (change lifecycle and the `cargo-workplan` CLI)." | prose |
| R21 | prose | body, ## Context | "It is opt-in so the default stays lightweight." | prose |

Kinds: checkbox, modal, list-item, scope-out, dependency, question, decision, prose.

## Candidate dispositions
| Candidate | Disposition |
|-----------|-------------|
| B1 | kept → R1 |
| B2 | kept → R2 |
| B3 | kept → R3 |
| B4 | kept → R4 |
| B5 | kept → R5 |
| B6 | kept → R6 |
| B7 | kept → R7 |
| B8 | kept → R8 |
| B9 | kept → R9 |
| B10 | kept → R10 |
| B11 | kept → R11 |
| B12 | kept → R12 |
| B13 | kept → R13 |
| B14 | kept → R14 |
| B15 | kept → R15 |
| B16 | kept → R16 |
| B17 | kept → R17 |
| B18 | kept → R18 |
| B19 | kept → R19 |

Dropped: 0. Prose additions: 2 (R20, R21).

## Comment coverage
| Comment | Author | Rows / disposition |
|---------|--------|--------------------|

The issue has 0 comments; there is nothing to cover.

## Dependencies (from brief.json relations)
- #4 mentions [state not recorded in brief.json]
- #8 mentions [CLOSED]
- #10 mentions [CLOSED]
- #3 2-hop [state not recorded in brief.json]
- #7 2-hop [state not recorded in brief.json]
- #9 2-hop [state not recorded in brief.json]

## NEEDS-JUDGMENT
1. R19 vs R3/R2: "Overlaps with #10: the two-way coverage and evidence-claims items there cover code-to-docs and tests-to-claims, which rules 3 and 4 above enforce under strict." / "Every Contract and Constraints claim is covered by a bound test or example.": the issue says scope should be reconciled with #10 so neither duplicates the other; the reconciliation is not stated.
2. R1: "Propose one location and justify it in the capability document." / "CLI: a `--strict` flag plus a repo-level setting (for example a field in a config file next to `docs/capabilities/index.md`, or `Cargo.toml` metadata).": the location of the repo-level setting is left as a choice among alternatives.
3. R7: "No `dropped` or `moved` item outcome without a stated reason field.": the name and shape of the reason field are not specified.
4. R9 vs #4 (brief.json): "Coordinate with the census debt-ratchet design (#4) instead of inventing a second ledger." / "A baseline/ratchet option so adopting strict on an existing repository can start from today's findings and fail only on new ones.": #4 is listed as "mentions" only in brief.json; whether its design exists and is binding is not established by this evidence.
5. R5 vs R14: "`change complete` additionally requires `verify` to pass for ALL bound items" / "Default (non-strict) behaviour and every existing fixture are unchanged.": it is not stated whether the R5 requirements apply only under strict.
6. R10 vs R11: "Reverse coverage mode (`cargo workplan coverage --reverse`, also enforced under strict with a threshold)" / "a configurable ceiling (maximum orphan lines or percentage per crate)": the threshold value and whether the ceiling default is set are not specified.
7. R18 vs R14: "Once the rules pass, the repository enables strict on its own docs." / "Default (non-strict) behaviour and every existing fixture are unchanged.": whether enabling strict on the repository's own docs is in scope for this change is left open by the wording.

## Self-check
Quotes checked: 21. Misses: 0. Comments covered: 0/0.
