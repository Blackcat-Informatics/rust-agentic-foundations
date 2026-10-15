# Requirements ledger: issue #8: raf-workplan change lifecycle + cargo-workplan CLI

Evidence: issue.md, requirements-candidates.md, brief.json (coverage: "issue #8 + 0 comment(s) via github"; "link crawl: 3 issue(s) within 2 hop(s) of `#N` references"; "requirement candidates: 17 line(s); 0 comment(s) matched no rule"; "referenced artifacts: 0 path(s) (0 on disk), 7 identifier(s), 1 workspace package(s), 0 URL(s)")

Line references: "body L<n>" is the line number in issue.md. The candidate file's "Line" column is body-relative and sits 7 lines before the issue.md line (candidate line 7 = issue.md line 14).

## Requirements
| R-id | Kind | Source | Quote (verbatim) | From candidate |
|------|------|--------|------------------|----------------|
| R1 | checkbox | body L14 | "`change new <slug> --capability cap:x` scaffolds the change record and a PROPOSED block" | B1 |
| R2 | checkbox | body L15 | "`change impact <slug>` lists the affected capabilities, sections, bindings and dependents" | B2 |
| R3 | checkbox | body L16 | "`change reconcile <slug>` produces the obligation list:" | B3 |
| R4 | list-item | body L17 | "unresolved proposals;" | B4 |
| R5 | list-item | body L18 | "items without an outcome;" | B5 |
| R6 | list-item | body L19 | "missing or broken bindings;" | B6 |
| R7 | list-item | body L20 | "affected sections not touched since the merge base (via the git CLI);" | B7 |
| R8 | list-item | body L21 | "stale generated blocks;" | B8 |
| R9 | list-item | body L22 | "future-tense or proposal text in a `delivered` capability." | B9 |
| R10 | checkbox | body L25 | "`change complete <slug>` requires zero obligations, every item resolved, no PROPOSED blocks for the change, `docs sync` clean, and bound tests passing (`cargo test` filtered to the bound tests). It then deletes the change file. A `contract_effect: none` change records that and carries lighter obligations." | B10 |
| R11 | checkbox | body L26 | "`verify [<slug>]` runs bound tests and examples and reports pass/fail (no persisted evidence in M1)" | B11 |
| R12 | checkbox | body L27 | "`docs preview <slug>` renders the capability as it will read once reconciled" | B12 |
| R13 | checkbox | body L28 | "`cargo-workplan` binary (`clap` derive) exposing `check`, `status`, `docs sync|preview`, `change new|impact|reconcile|complete`, `verify`; exit codes 0/1/2" | B13 |
| R14 | checkbox | body L31 | "Each obligation kind has a fixture that triggers it, and `complete` refuses in each case" | B14 |
| R15 | checkbox | body L32 | "A happy-path fixture completes and the change file is gone" | B15 |
| R16 | checkbox | body L33 | "CLI integration tests" | B16 |
| R17 | checkbox | body L34 | "Capability docs reconciled" | B17 |
| R18 | prose | body L9 | "A change names the documentation it affects when it is planned." | prose |
| R19 | prose | body L9 | "`reconcile` produces a precise, reviewable obligation list that humans or agents resolve by writing prose." | prose |
| R20 | modal | body L9 | "`complete` refuses until the change has been incorporated into the current account of the system" | prose |
| R21 | prose | body L9 | "then **deletes the change record**, leaving its history in Git and off the reading path." | prose |
| R22 | prose | body L11 | "Capabilities delivered: `cap:workplan-reconcile`, `cap:workplan-complete-gate`, `cap:workplan-cli`." | prose |
| R23 | prose | body L24 | "Each obligation names the evidence needed to resolve it." | prose |
| R24 | dependency | body L37 | "- #7" | prose |

Notes on R-rows:
- R4 to R9 are the sub-bullets under B3 (body L17 to L22). Each is a separate obligation kind and each needs its own fixture under R14.
- R10 keeps the `contract_effect: none` sentence from the same line; it is not split out as a separate row.
- R20 and R21 quote the Context paragraph, which describes the same lifecycle that R10 specifies. They are kept as separate rows and are cross-referenced in NEEDS-JUDGMENT item 1.
- R23 and R22 were not matched by any lexical rule, so they are prose additions.
- R24 is taken from the "Depends on" section. The brief classifies #7 as "mentions" and lists it as CLOSED, which is a different relation from the issue's own "Depends on" heading.

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

Dropped candidates: none. Prose additions: 7 (R18 to R24).

## Comment coverage
| Comment | Author | Rows / disposition |
|---------|--------|--------------------|
| (none) | - | The issue has 0 comments (`## Comments (0)` at issue.md L39; brief.json coverage "0 comment(s)"). Comments covered: 0/0. |

## Dependencies (from brief.json relations)
- #7 mentions [CLOSED] ("raf-workplan model: documents, check, graph, bindings, generated docs"). The issue body also lists it under "Depends on" (R24).
- #1 2-hop [state not recorded in brief.json] (linked by the crawl; not named in the issue body)
- #2 2-hop [state not recorded in brief.json] (linked by the crawl; not named in the issue body)

## NEEDS-JUDGMENT
1. R20 vs R10: "`complete` refuses until the change has been incorporated into the current account of the system" / "`change complete <slug>` requires zero obligations, every item resolved, no PROPOSED blocks for the change, `docs sync` clean, and bound tests passing": the Context paragraph gives an undefined test ("incorporated into the current account of the system") and the Scope line gives a list of checks. Is the list the full definition of "incorporated", or is there a further condition?
2. R10 vs R11: "`change complete <slug>` requires ... bound tests passing" / "`verify [<slug>]` runs bound tests and examples and reports pass/fail (no persisted evidence in M1)": `complete` requires passing bound tests, but nothing is persisted. Does `complete` run the tests itself, or does it check a result recorded elsewhere? The issue does not say.
3. R10: "A `contract_effect: none` change records that and carries lighter obligations.": the issue does not define which obligations are lighter, where "records that" is stored, or what "contract_effect" values exist. The meaning depends on a choice that the issue has not made.
4. R13: "exit codes 0/1/2": the issue does not say which condition maps to 0, 1, or 2.
5. R21 vs R10: "then **deletes the change record**" (R21) / "It then deletes the change file." (R10): two different nouns, "record" and "file", for the same deleted artifact. Confirm that these name the same thing.
6. R3 vs R7: "affected sections not touched since the merge base (via the git CLI)" is an obligation that depends on the state of the git repository at reconcile time. The issue does not say what counts as "touched" or which merge base is used when the branch has none.

## Self-check
Quotes checked: 24. Comments covered: 0/0.
Quote misses: 0
