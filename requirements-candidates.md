# Requirement candidates — issue #8

Lines from the issue body and comments that matched a lexical rule, copied
verbatim with the rule that fired. This is a CANDIDATE list: rules both
over-match (a "should" in passing) and under-match (a requirement stated
without a keyword). The issue-analyzer keeps or drops each row, adds what
the rules missed, and accounts for every silent comment. Code fences are
skipped.

| ID | Source | Line | Rules | Text |
|---|---|---|---|---|
| B1 | body | 7 | checkbox | - [ ] `change new <slug> --capability cap:x` scaffolds the change record and a PROPOSED block |
| B2 | body | 8 | checkbox | - [ ] `change impact <slug>` lists the affected capabilities, sections, bindings and dependents |
| B3 | body | 9 | checkbox | - [ ] `change reconcile <slug>` produces the obligation list: |
| B4 | body | 10 | list-under-requirement-heading | - unresolved proposals; |
| B5 | body | 11 | list-under-requirement-heading | - items without an outcome; |
| B6 | body | 12 | list-under-requirement-heading | - missing or broken bindings; |
| B7 | body | 13 | list-under-requirement-heading | - affected sections not touched since the merge base (via the git CLI); |
| B8 | body | 14 | list-under-requirement-heading | - stale generated blocks; |
| B9 | body | 15 | list-under-requirement-heading | - future-tense or proposal text in a `delivered` capability. |
| B10 | body | 18 | checkbox | - [ ] `change complete <slug>` requires zero obligations, every item resolved, no PROPOSED blocks for the change, `docs sync` clean, and bound tests passing (`cargo test` filtered to the bound tests). It then deletes the change file. A `contract_effect: none` change records that and carries lighter obligations. |
| B11 | body | 19 | checkbox | - [ ] `verify [<slug>]` runs bound tests and examples and reports pass/fail (no persisted evidence in M1) |
| B12 | body | 20 | checkbox | - [ ] `docs preview <slug>` renders the capability as it will read once reconciled |
| B13 | body | 21 | checkbox | - [ ] `cargo-workplan` binary (`clap` derive) exposing `check`, `status`, `docs sync\|preview`, `change new\|impact\|reconcile\|complete`, `verify`; exit codes 0/1/2 |
| B14 | body | 24 | checkbox | - [ ] Each obligation kind has a fixture that triggers it, and `complete` refuses in each case |
| B15 | body | 25 | checkbox | - [ ] A happy-path fixture completes and the change file is gone |
| B16 | body | 26 | checkbox | - [ ] CLI integration tests |
| B17 | body | 27 | checkbox | - [ ] Capability docs reconciled |

## Comments that matched no rule

None.
